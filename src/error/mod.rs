use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum VrcError {
    #[error("invalid API URL: {0}")]
    Url(#[from] url::ParseError),

    #[error("request failed: {0}")]
    Network(reqwest::Error),

    #[error("response decode failed: {0}")]
    Decode(String),

    #[error("query encode failed: {0}")]
    Query(#[from] serde_urlencoded::ser::Error),

    #[error("API returned {status}: {body}")]
    Api { status: StatusCode, body: String },

    #[error("authentication failed: {0}")]
    Auth(String),
}
