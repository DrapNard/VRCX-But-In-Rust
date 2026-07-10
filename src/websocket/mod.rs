pub mod event;

use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Error as TungsteniteError, Message,
        client::IntoClientRequest,
        http::{HeaderValue, header::USER_AGENT},
    },
};
use url::Url;

pub use event::{PipelineEvent, PipelineMessage, PipelineParseError, parse_pipeline_message};

const DEFAULT_PIPELINE_URL: &str = "wss://pipeline.vrchat.cloud/";

pub type PipelineSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct PipelineClient {
    pipeline_url: Url,
    auth_token: String,
    user_agent: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("invalid websocket URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("invalid user-agent header: {0}")]
    InvalidUserAgent(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),

    #[error("websocket error: {0}")]
    WebSocket(#[from] TungsteniteError),

    #[error("pipeline message parse error: {0}")]
    Parse(#[from] PipelineParseError),

    #[error("pipeline emitted an error: {0}")]
    Remote(String),

    #[error("websocket closed")]
    Closed,
}

impl PipelineClient {
    pub fn new(
        auth_token: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Result<Self, PipelineError> {
        Self::with_url(DEFAULT_PIPELINE_URL, auth_token, user_agent)
    }

    pub fn with_url(
        pipeline_url: &str,
        auth_token: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Result<Self, PipelineError> {
        Ok(Self {
            pipeline_url: Url::parse(pipeline_url)?,
            auth_token: auth_token.into(),
            user_agent: user_agent.into(),
        })
    }

    pub async fn connect(&self) -> Result<PipelineConnection, PipelineError> {
        let mut url = self.pipeline_url.clone();
        url.query_pairs_mut()
            .append_pair("authToken", &self.auth_token);

        tracing::debug!(
            scheme = self.pipeline_url.scheme(),
            host = self.pipeline_url.host_str().unwrap_or_default(),
            "connecting websocket"
        );
        let mut request = url.as_str().into_client_request()?;
        request
            .headers_mut()
            .insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);

        let (socket, _) = connect_async(request).await?;

        tracing::info!("websocket connected");
        Ok(PipelineConnection { socket })
    }
}

#[derive(Debug)]
pub struct PipelineConnection {
    socket: PipelineSocket,
}

impl PipelineConnection {
    pub async fn next_message(&mut self) -> Result<PipelineMessage, PipelineError> {
        loop {
            let message = self.socket.next().await.ok_or(PipelineError::Closed)??;

            match message {
                Message::Text(text) => {
                    tracing::trace!(bytes = text.len(), "websocket text message");
                    let parsed = parse_pipeline_message(text.as_ref())?;

                    if let PipelineMessage::Error(error) = &parsed {
                        return Err(PipelineError::Remote(error.err.clone()));
                    }

                    return Ok(parsed);
                }
                Message::Binary(bytes) => {
                    tracing::trace!(bytes = bytes.len(), "websocket binary message");
                    let text = std::str::from_utf8(&bytes).map_err(|err| {
                        PipelineParseError::Json(format!("binary message is not UTF-8: {err}"))
                    })?;
                    let parsed = parse_pipeline_message(text)?;

                    if let PipelineMessage::Error(error) = &parsed {
                        return Err(PipelineError::Remote(error.err.clone()));
                    }

                    return Ok(parsed);
                }
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
                Message::Close(frame) => {
                    tracing::warn!(?frame, "websocket closed by remote");
                    return Err(PipelineError::Closed);
                }
            }
        }
    }

    pub fn into_inner(self) -> PipelineSocket {
        self.socket
    }
}
