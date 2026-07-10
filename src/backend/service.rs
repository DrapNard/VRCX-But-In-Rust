use std::{path::PathBuf, sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, broadcast, watch},
    task::JoinHandle,
};
use url::Url;

use crate::{
    api::{PaginationQuery, friends::FriendsQuery},
    client::VrcClient,
    error::VrcError,
    models::users::User,
    session::auth::{
        Auth, AuthError, LoginResult, LogoutResult, RestoreSessionResult, TwoFactorMethod,
        VerifyTwoFactorResult,
    },
    store::{AppSnapshot, AppStore, CacheConfig, SessionMetadata, StoreError, WebSocketStatus},
    websocket::{PipelineClient, PipelineError, PipelineMessage},
};

#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub api_base_url: Url,
    pub pipeline_url: Url,
    pub database_path: PathBuf,
    pub user_agent: String,
    pub cache: CacheConfig,
    pub reconnect_min_delay: Duration,
    pub reconnect_max_delay: Duration,
}

impl BackendConfig {
    pub fn new(database_path: impl Into<PathBuf>) -> Self {
        Self {
            api_base_url: Url::parse("https://api.vrchat.cloud/api/1/").unwrap(),
            pipeline_url: Url::parse("wss://pipeline.vrchat.cloud/").unwrap(),
            database_path: database_path.into(),
            user_agent: "vrcx-rs/0.1".to_string(),
            cache: CacheConfig::default(),
            reconnect_min_delay: Duration::from_secs(1),
            reconnect_max_delay: Duration::from_secs(30),
        }
    }

    pub fn for_app() -> Result<Self, BackendError> {
        let directories = directories::ProjectDirs::from("dev", "vrcx-rs", "VRCX Rust")
            .ok_or_else(|| BackendError::Config("application data directory unavailable".into()))?;
        std::fs::create_dir_all(directories.data_local_dir())
            .map_err(|error| BackendError::Config(error.to_string()))?;
        Ok(Self::new(directories.data_local_dir().join("cache.redb")))
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    pub user_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub enum LoginOutcome {
    Authenticated(AuthenticatedSession),
    TwoFactorRequired(Vec<TwoFactorMethod>),
    InvalidCredentials,
}

#[derive(Debug, Clone)]
pub enum BackendEvent {
    SessionRestored(AuthenticatedSession),
    Authenticated(AuthenticatedSession),
    TwoFactorRequired(Vec<TwoFactorMethod>),
    SyncCompleted,
    SyncFailed { reason: String },
    WebSocketConnected,
    WebSocketDisconnected { reason: String },
    LoggedOut,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("backend configuration error: {0}")]
    Config(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("API error: {0}")]
    Api(#[from] VrcError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("websocket setup error: {0}")]
    WebSocket(#[from] PipelineError),
    #[error("backend task failed: {0}")]
    Task(String),
}

pub struct Backend {
    auth: Arc<Auth>,
    api: VrcClient,
    store: AppStore,
    config: BackendConfig,
    events: broadcast::Sender<BackendEvent>,
    websocket: Mutex<Option<WebSocketRuntime>>,
}

struct WebSocketRuntime {
    stop: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl Backend {
    pub fn open(config: BackendConfig) -> Result<Self, BackendError> {
        let auth = Arc::new(
            Auth::new(config.api_base_url.clone())
                .map_err(|error| BackendError::Auth(error.to_string()))?,
        );
        let api = VrcClient::from_auth(&auth);
        let store = AppStore::open(&config.database_path, config.cache.clone())?;
        let (events, _) = broadcast::channel(128);

        Ok(Self {
            auth,
            api,
            store,
            config,
            events,
            websocket: Mutex::new(None),
        })
    }

    pub fn api(&self) -> &VrcClient {
        &self.api
    }

    pub fn store(&self) -> &AppStore {
        &self.store
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<BackendEvent> {
        self.events.subscribe()
    }

    pub fn subscribe_state(&self) -> watch::Receiver<AppSnapshot> {
        self.store.subscribe()
    }

    pub async fn restore_session(&self) -> Result<Option<AuthenticatedSession>, BackendError> {
        match self.auth.restore_session().await {
            RestoreSessionResult::Success(user) => {
                let session = session_from_user(&user);
                self.activate_session(user).await?;
                let _ = self
                    .events
                    .send(BackendEvent::SessionRestored(session.clone()));
                Ok(Some(session))
            }
            RestoreSessionResult::NoSavedSession | RestoreSessionResult::InvalidSession => Ok(None),
            RestoreSessionResult::NetworkError(error)
            | RestoreSessionResult::DecodeError(error) => {
                Err(BackendError::Auth(error.to_string()))
            }
            RestoreSessionResult::SessionLoadError(error) => Err(auth_error(error)),
            RestoreSessionResult::InvalidUrl => {
                Err(BackendError::Auth("invalid authentication URL".to_string()))
            }
        }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginOutcome, BackendError> {
        match self.auth.login(username, password).await {
            LoginResult::Success(user) => {
                let session = session_from_user(&user);
                self.activate_session(user).await?;
                let _ = self
                    .events
                    .send(BackendEvent::Authenticated(session.clone()));
                Ok(LoginOutcome::Authenticated(session))
            }
            LoginResult::TwoFactorRequired(methods) => {
                let _ = self
                    .events
                    .send(BackendEvent::TwoFactorRequired(methods.clone()));
                Ok(LoginOutcome::TwoFactorRequired(methods))
            }
            LoginResult::InvalidCredentials => Ok(LoginOutcome::InvalidCredentials),
            LoginResult::HttpError(status) => {
                Err(BackendError::Auth(format!("HTTP status {status}")))
            }
            LoginResult::NetworkError(error) | LoginResult::DecodeError(error) => {
                Err(BackendError::Auth(error.to_string()))
            }
            LoginResult::SessionSaveError(error) => Err(auth_error(error)),
            LoginResult::InvalidUrl => {
                Err(BackendError::Auth("invalid authentication URL".to_string()))
            }
        }
    }

    pub async fn verify_two_factor(
        &self,
        method: TwoFactorMethod,
        code: &str,
    ) -> Result<LoginOutcome, BackendError> {
        match self.auth.verify_2fa(method, code).await {
            VerifyTwoFactorResult::Success(user) => {
                let session = session_from_user(&user);
                self.activate_session(user).await?;
                let _ = self
                    .events
                    .send(BackendEvent::Authenticated(session.clone()));
                Ok(LoginOutcome::Authenticated(session))
            }
            VerifyTwoFactorResult::InvalidCode => Ok(LoginOutcome::InvalidCredentials),
            VerifyTwoFactorResult::HttpError(status) => {
                Err(BackendError::Auth(format!("HTTP status {status}")))
            }
            VerifyTwoFactorResult::NetworkError(error)
            | VerifyTwoFactorResult::DecodeError(error) => {
                Err(BackendError::Auth(error.to_string()))
            }
            VerifyTwoFactorResult::SessionSaveError(error) => Err(auth_error(error)),
            VerifyTwoFactorResult::InvalidUrl => {
                Err(BackendError::Auth("invalid two-factor URL".to_string()))
            }
        }
    }

    pub async fn synchronize(&self) -> Result<(), BackendError> {
        let (online, offline, notifications) = tokio::join!(
            self.fetch_all_friends(false),
            self.fetch_all_friends(true),
            self.fetch_all_notifications()
        );

        let mut friends = online?;
        friends.extend(offline?);
        self.store.replace_friends(friends).await?;
        self.store.replace_notifications(notifications?).await?;
        self.store.touch_last_sync().await?;
        let _ = self.events.send(BackendEvent::SyncCompleted);
        Ok(())
    }

    pub async fn logout(&self) -> Result<(), BackendError> {
        self.stop_websocket().await?;
        match self.auth.logout().await {
            LogoutResult::Success | LogoutResult::AlreadyLoggedOut => {}
            LogoutResult::HttpError(status) => {
                return Err(BackendError::Auth(format!("logout HTTP status {status}")));
            }
            LogoutResult::NetworkError(error) => {
                return Err(BackendError::Auth(error.to_string()));
            }
            LogoutResult::InvalidUrl => {
                return Err(BackendError::Auth("invalid logout URL".to_string()));
            }
        }
        self.store.clear_session().await?;
        let _ = self.events.send(BackendEvent::LoggedOut);
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), BackendError> {
        self.stop_websocket().await
    }

    async fn activate_session(&self, user: User) -> Result<(), BackendError> {
        self.store.cache().put_user(user.clone()).await?;
        self.store
            .set_session_metadata(SessionMetadata {
                user_id: Some(user.identity.id.clone()),
                display_name: Some(user.identity.display_name.clone()),
                last_sync_unix_ms: None,
                websocket_status: WebSocketStatus::Disconnected,
                websocket_error: None,
            })
            .await?;
        self.start_websocket().await?;
        if let Err(error) = self.synchronize().await {
            let _ = self.events.send(BackendEvent::SyncFailed {
                reason: error.to_string(),
            });
        }
        Ok(())
    }

    async fn fetch_all_friends(
        &self,
        offline: bool,
    ) -> Result<Vec<crate::models::friend::Friend>, VrcError> {
        let mut offset = 0;
        let mut all = Vec::new();
        loop {
            let page = self
                .api
                .friends(
                    FriendsQuery::new()
                        .limit(100)
                        .offset(offset)
                        .offline(offline),
                )
                .await?;
            let count = page.len();
            all.extend(page);
            if count < 100 {
                return Ok(all);
            }
            offset += count as u32;
        }
    }

    async fn fetch_all_notifications(
        &self,
    ) -> Result<Vec<crate::models::notification::NotificationV2>, VrcError> {
        let mut offset = 0;
        let mut all = Vec::new();
        loop {
            let page = self
                .api
                .notifications(&PaginationQuery::new().limit(100).offset(offset))
                .await?;
            let count = page.items.len();
            let total = page.total_count as usize;
            all.extend(page.items);
            if count < 100 || all.len() >= total {
                return Ok(all);
            }
            offset += count as u32;
        }
    }

    async fn start_websocket(&self) -> Result<(), BackendError> {
        self.stop_websocket().await?;
        let auth_token = self.auth.auth_token().map_err(auth_error)?;
        let pipeline_url = self.config.pipeline_url.to_string();
        let user_agent = self.config.user_agent.clone();
        let api = self.api.clone();
        let store = self.store.clone();
        let events = self.events.clone();
        let min_delay = self.config.reconnect_min_delay;
        let max_delay = self.config.reconnect_max_delay;
        let (stop, stop_rx) = watch::channel(false);

        let task = tokio::spawn(async move {
            websocket_loop(
                pipeline_url,
                auth_token,
                user_agent,
                api,
                store,
                events,
                stop_rx,
                min_delay,
                max_delay,
            )
            .await;
        });
        *self.websocket.lock().await = Some(WebSocketRuntime { stop, task });
        Ok(())
    }

    async fn stop_websocket(&self) -> Result<(), BackendError> {
        let runtime = self.websocket.lock().await.take();
        if let Some(runtime) = runtime {
            let _ = runtime.stop.send(true);
            runtime
                .task
                .await
                .map_err(|error| BackendError::Task(error.to_string()))?;
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn websocket_loop(
    pipeline_url: String,
    auth_token: String,
    user_agent: String,
    api: VrcClient,
    store: AppStore,
    events: broadcast::Sender<BackendEvent>,
    mut stop: watch::Receiver<bool>,
    min_delay: Duration,
    max_delay: Duration,
) {
    let mut delay = min_delay;
    loop {
        if *stop.borrow() {
            break;
        }
        let _ = store
            .set_websocket_status_with_error(WebSocketStatus::Connecting, None)
            .await;
        tracing::info!("connecting to VRChat pipeline");
        let client =
            match PipelineClient::with_url(&pipeline_url, auth_token.clone(), user_agent.clone()) {
                Ok(client) => client,
                Err(error) => {
                    let reason = error.to_string();
                    tracing::error!(%reason, "websocket client setup failed");
                    let _ = store
                        .set_websocket_status_with_error(
                            WebSocketStatus::Disconnected,
                            Some(reason.clone()),
                        )
                        .await;
                    let _ = events.send(BackendEvent::WebSocketDisconnected { reason });
                    break;
                }
            };

        let connection = tokio::select! {
            result = client.connect() => result,
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    break;
                }
                continue;
            }
        };

        match connection {
            Ok(mut connection) => {
                tracing::info!("VRChat pipeline is live");
                delay = min_delay;
                let _ = store
                    .set_websocket_status_with_error(WebSocketStatus::Connected, None)
                    .await;
                let _ = events.send(BackendEvent::WebSocketConnected);
                if let Err(error) = resynchronize(&api, &store).await {
                    let _ = events.send(BackendEvent::SyncFailed {
                        reason: error.to_string(),
                    });
                } else {
                    let _ = events.send(BackendEvent::SyncCompleted);
                }
                loop {
                    tokio::select! {
                        changed = stop.changed() => {
                            if changed.is_err() || *stop.borrow() {
                                let _ = store
                                    .set_websocket_status_with_error(WebSocketStatus::Disconnected, None)
                                    .await;
                                return;
                            }
                        }
                        message = connection.next_message() => {
                            match message {
                                Ok(PipelineMessage::Event(event)) => {
                                    if let Err(error) = store.apply_pipeline_event(event).await {
                                        let reason = error.to_string();
                                        tracing::error!(%reason, "pipeline event application failed");
                                        let _ = events.send(BackendEvent::SyncFailed {
                                            reason,
                                        });
                                    }
                                }
                                Ok(PipelineMessage::Error(error)) => {
                                    let _ = store
                                        .set_websocket_status_with_error(
                                            WebSocketStatus::Disconnected,
                                            Some(error.err.clone()),
                                        )
                                        .await;
                                    let _ = events.send(BackendEvent::WebSocketDisconnected {
                                        reason: error.err,
                                    });
                                    break;
                                }
                                Err(error) => {
                                    let reason = error.to_string();
                                    tracing::warn!(%reason, "websocket connection lost");
                                    let _ = store
                                        .set_websocket_status_with_error(
                                            WebSocketStatus::Disconnected,
                                            Some(reason.clone()),
                                        )
                                        .await;
                                    let _ = events.send(BackendEvent::WebSocketDisconnected {
                                        reason,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => {
                let reason = error.to_string();
                tracing::warn!(%reason, "websocket connection failed");
                let _ = store
                    .set_websocket_status_with_error(
                        WebSocketStatus::Disconnected,
                        Some(reason.clone()),
                    )
                    .await;
                let _ = events.send(BackendEvent::WebSocketDisconnected { reason });
            }
        }
        tracing::info!(
            delay_ms = delay.as_millis(),
            "scheduling websocket reconnect"
        );

        if store.snapshot().await.session.websocket_error.is_none() {
            let _ = store
                .set_websocket_status_with_error(WebSocketStatus::Disconnected, None)
                .await;
        }
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    break;
                }
            }
        }
        delay = delay.saturating_mul(2).min(max_delay);
    }
    let _ = store
        .set_websocket_status(WebSocketStatus::Disconnected)
        .await;
}

async fn resynchronize(api: &VrcClient, store: &AppStore) -> Result<(), BackendError> {
    async fn friends(
        api: &VrcClient,
        offline: bool,
    ) -> Result<Vec<crate::models::friend::Friend>, VrcError> {
        let mut offset = 0;
        let mut all = Vec::new();
        loop {
            let page = api
                .friends(
                    FriendsQuery::new()
                        .limit(100)
                        .offset(offset)
                        .offline(offline),
                )
                .await?;
            let count = page.len();
            all.extend(page);
            if count < 100 {
                return Ok(all);
            }
            offset += count as u32;
        }
    }

    async fn notifications(
        api: &VrcClient,
    ) -> Result<Vec<crate::models::notification::NotificationV2>, VrcError> {
        let mut offset = 0;
        let mut all = Vec::new();
        loop {
            let page = api
                .notifications(&PaginationQuery::new().limit(100).offset(offset))
                .await?;
            let count = page.items.len();
            let total = page.total_count as usize;
            all.extend(page.items);
            if count < 100 || all.len() >= total {
                return Ok(all);
            }
            offset += count as u32;
        }
    }

    let (online, offline, notifications) =
        tokio::join!(friends(api, false), friends(api, true), notifications(api));
    let mut friends = online?;
    friends.extend(offline?);
    store.replace_friends(friends).await?;
    store.replace_notifications(notifications?).await?;
    store.touch_last_sync().await?;
    Ok(())
}

fn session_from_user(user: &User) -> AuthenticatedSession {
    AuthenticatedSession {
        user_id: user.identity.id.clone(),
        display_name: user.identity.display_name.clone(),
    }
}

fn auth_error(error: AuthError) -> BackendError {
    BackendError::Auth(format!("{error:?}"))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn database_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vrcx-backend-{name}-{}-{}.redb",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn opens_subscribes_and_shuts_down_without_a_session() {
        let path = database_path("lifecycle");
        let backend = Backend::open(BackendConfig::new(&path)).unwrap();
        let state = backend.subscribe_state();

        assert!(state.borrow().session.user_id.is_none());
        backend.shutdown().await.unwrap();

        drop(state);
        drop(backend);
        std::fs::remove_file(path).ok();
    }
}
