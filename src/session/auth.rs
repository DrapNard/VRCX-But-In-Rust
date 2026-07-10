use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{
    Client, StatusCode,
    cookie::{CookieStore, Jar},
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::models::users::User;
use crate::session::tokens::SessionTokens;

#[derive(Debug)]
pub struct Auth {
    base_url: Url,
    client: Client,
    cookie_jar: Arc<Jar>,
    keyring_service: String,
    keyring_account: String,
    user_agent: String,
}

#[derive(Debug)]
pub enum LoginResult {
    Success(User),
    TwoFactorRequired(Vec<TwoFactorMethod>),
    InvalidCredentials,
    HttpError(StatusCode),
    NetworkError(reqwest::Error),
    DecodeError(reqwest::Error),
    SessionSaveError(AuthError),
    InvalidUrl,
}

#[derive(Debug)]
pub enum RestoreSessionResult {
    Success(User),
    NoSavedSession,
    InvalidSession,
    NetworkError(reqwest::Error),
    DecodeError(reqwest::Error),
    SessionLoadError(AuthError),
    InvalidUrl,
}

#[derive(Debug)]
pub enum VerifyTwoFactorResult {
    Success(User),
    InvalidCode,
    HttpError(StatusCode),
    NetworkError(reqwest::Error),
    DecodeError(reqwest::Error),
    SessionSaveError(AuthError),
    InvalidUrl,
}

#[derive(Debug)]
pub enum AuthError {
    Keyring(keyring::Error),
    Serde(serde_json::Error),
    MissingAuthCookie,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwoFactorMethod {
    Totp,
    EmailOtp,
    RecoveryCode,
    Unknown(String),
}

#[derive(Debug)]
pub enum LogoutResult {
    Success,
    AlreadyLoggedOut,
    HttpError(StatusCode),
    NetworkError(reqwest::Error),
    InvalidUrl,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredSession {
    auth: String,

    #[serde(rename = "twoFactorAuth")]
    two_factor_auth: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerifyTwoFactorBody<'a> {
    code: &'a str,
}

#[derive(Debug, Deserialize)]
struct VerifyTwoFactorResponse {
    verified: Option<bool>,
}

impl Auth {
    pub fn new(base_url: Url) -> Result<Self, reqwest::Error> {
        let cookie_jar = Arc::new(Jar::default());

        let user_agent = "vrcx-rs/0.1".to_string();

        let client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .user_agent(&user_agent)
            .build()?;

        Ok(Self {
            base_url,
            client,
            cookie_jar,
            keyring_service: "vrcx-rs".to_string(),
            keyring_account: "vrchat-session".to_string(),
            user_agent,
        })
    }

    pub async fn login(&self, username: &str, password: &str) -> LoginResult {
        tracing::info!("starting authentication");
        let authorization = build_basic_auth(username, password);

        let url = match self.endpoint("auth/user") {
            Ok(url) => url,
            Err(_) => return LoginResult::InvalidUrl,
        };

        let response = match self
            .client
            .get(url)
            .header("Authorization", authorization)
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) => {
                tracing::error!(error = %err, "authentication request failed");
                return LoginResult::NetworkError(err);
            }
        };

        let status = response.status();
        tracing::debug!(%status, "authentication response");

        if status == StatusCode::UNAUTHORIZED {
            tracing::warn!("authentication rejected");
            return LoginResult::InvalidCredentials;
        }

        if !status.is_success() {
            return LoginResult::HttpError(status);
        }

        let user = match response.json::<User>().await {
            Ok(user) => user,
            Err(err) => return LoginResult::DecodeError(err),
        };

        let two_factor_methods = detect_two_factor_methods(&user);

        if !two_factor_methods.is_empty() {
            tracing::info!("two-factor authentication required");
            return LoginResult::TwoFactorRequired(two_factor_methods);
        }

        if let Err(err) = self.save_session() {
            return LoginResult::SessionSaveError(err);
        }

        tracing::info!("authentication succeeded");
        LoginResult::Success(user)
    }

    pub async fn verify_2fa(&self, method: TwoFactorMethod, code: &str) -> VerifyTwoFactorResult {
        tracing::info!(?method, "verifying two-factor authentication");
        let endpoint = match method {
            TwoFactorMethod::Totp => "auth/twofactorauth/totp/verify",
            TwoFactorMethod::EmailOtp => "auth/twofactorauth/emailotp/verify",
            TwoFactorMethod::RecoveryCode => "auth/twofactorauth/otp/verify",
            TwoFactorMethod::Unknown(_) => return VerifyTwoFactorResult::InvalidUrl,
        };

        let url = match self.endpoint(endpoint) {
            Ok(url) => url,
            Err(_) => return VerifyTwoFactorResult::InvalidUrl,
        };

        let response = match self
            .client
            .post(url)
            .json(&VerifyTwoFactorBody { code })
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) => return VerifyTwoFactorResult::NetworkError(err),
        };

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return VerifyTwoFactorResult::InvalidCode;
        }

        if !status.is_success() {
            return VerifyTwoFactorResult::HttpError(status);
        }

        let verify_response = match response.json::<VerifyTwoFactorResponse>().await {
            Ok(body) => body,
            Err(err) => return VerifyTwoFactorResult::DecodeError(err),
        };

        if !verify_response.verified.unwrap_or(false) {
            tracing::warn!("two-factor authentication rejected");
            return VerifyTwoFactorResult::InvalidCode;
        }

        let user = match self.current_user().await {
            Ok(user) => user,
            Err(CurrentUserError::Network(err)) => {
                return VerifyTwoFactorResult::NetworkError(err);
            }
            Err(CurrentUserError::Decode(err)) => {
                return VerifyTwoFactorResult::DecodeError(err);
            }
            Err(CurrentUserError::InvalidUrl) => {
                return VerifyTwoFactorResult::InvalidUrl;
            }
            Err(CurrentUserError::Http(status)) => {
                return VerifyTwoFactorResult::HttpError(status);
            }
        };

        if let Err(err) = self.save_session() {
            return VerifyTwoFactorResult::SessionSaveError(err);
        }

        tracing::info!("two-factor authentication succeeded");
        VerifyTwoFactorResult::Success(user)
    }

    pub async fn restore_session(&self) -> RestoreSessionResult {
        tracing::info!("restoring saved session");
        let stored_session = match self.load_session() {
            Ok(Some(session)) => session,
            Ok(None) => return RestoreSessionResult::NoSavedSession,
            Err(err) => return RestoreSessionResult::SessionLoadError(err),
        };

        self.inject_session_cookies(&stored_session);

        match self.current_user().await {
            Ok(user) => {
                tracing::info!("saved session restored");
                RestoreSessionResult::Success(user)
            }

            Err(CurrentUserError::Http(StatusCode::UNAUTHORIZED))
            | Err(CurrentUserError::Http(StatusCode::FORBIDDEN)) => {
                tracing::warn!("saved session is no longer valid");
                let _ = self.clear_saved_session();
                RestoreSessionResult::InvalidSession
            }

            Err(CurrentUserError::Http(_)) => RestoreSessionResult::InvalidSession,

            Err(CurrentUserError::Network(err)) => RestoreSessionResult::NetworkError(err),

            Err(CurrentUserError::Decode(err)) => RestoreSessionResult::DecodeError(err),

            Err(CurrentUserError::InvalidUrl) => RestoreSessionResult::InvalidUrl,
        }
    }

    pub fn save_session(&self) -> Result<(), AuthError> {
        let cookies = self.extract_session_cookies()?;

        let json = serde_json::to_string(&cookies).map_err(AuthError::Serde)?;

        let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_account)
            .map_err(AuthError::Keyring)?;

        entry.set_password(&json).map_err(AuthError::Keyring)?;

        Ok(())
    }

    fn load_session(&self) -> Result<Option<StoredSession>, AuthError> {
        let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_account)
            .map_err(AuthError::Keyring)?;

        let json = match entry.get_password() {
            Ok(json) => json,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(err) => return Err(AuthError::Keyring(err)),
        };

        let session = serde_json::from_str::<StoredSession>(&json).map_err(AuthError::Serde)?;

        Ok(Some(session))
    }

    pub fn clear_saved_session(&self) -> Result<(), AuthError> {
        let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_account)
            .map_err(AuthError::Keyring)?;

        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(AuthError::Keyring(err)),
        }
    }

    pub fn session_tokens(&self) -> Result<SessionTokens, AuthError> {
        let session = self.extract_session_cookies()?;

        Ok(SessionTokens {
            auth: Some(session.auth),
            two_factor_auth: session.two_factor_auth,
        })
    }

    pub fn auth_token(&self) -> Result<String, AuthError> {
        self.extract_session_cookies().map(|session| session.auth)
    }

    fn clear_runtime_cookies(&self) {
        self.cookie_jar
            .add_cookie_str("auth=; Path=/; Max-Age=0; Secure; HttpOnly", &self.base_url);

        self.cookie_jar.add_cookie_str(
            "twoFactorAuth=; Path=/; Max-Age=0; Secure; HttpOnly",
            &self.base_url,
        );
    }

    pub async fn logout(&self) -> LogoutResult {
        tracing::info!("logging out");
        let url = match self.endpoint("logout") {
            Ok(url) => url,
            Err(_) => return LogoutResult::InvalidUrl,
        };

        let response = match self.client.put(url).send().await {
            Ok(response) => response,
            Err(err) => return LogoutResult::NetworkError(err),
        };

        let status = response.status();

        // Même si VRChat répond 401, on nettoie quand même la session locale.
        let _ = self.clear_saved_session();
        self.clear_runtime_cookies();

        if status.is_success() {
            LogoutResult::Success
        } else if status == StatusCode::UNAUTHORIZED {
            LogoutResult::AlreadyLoggedOut
        } else {
            LogoutResult::HttpError(status)
        }
    }

    async fn current_user(&self) -> Result<User, CurrentUserError> {
        let url = self
            .endpoint("auth/user")
            .map_err(|_| CurrentUserError::InvalidUrl)?;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(CurrentUserError::Network)?;

        let status = response.status();

        if !status.is_success() {
            return Err(CurrentUserError::Http(status));
        }

        response
            .json::<User>()
            .await
            .map_err(CurrentUserError::Decode)
    }

    fn endpoint(&self, path: &str) -> Result<Url, url::ParseError> {
        self.base_url.join(path)
    }

    fn extract_session_cookies(&self) -> Result<StoredSession, AuthError> {
        let cookie_header = self
            .cookie_jar
            .cookies(&self.base_url)
            .ok_or(AuthError::MissingAuthCookie)?;

        let cookie_header = cookie_header
            .to_str()
            .map_err(|_| AuthError::MissingAuthCookie)?;

        let auth = find_cookie(cookie_header, "auth").ok_or(AuthError::MissingAuthCookie)?;

        let two_factor_auth = find_cookie(cookie_header, "twoFactorAuth");

        Ok(StoredSession {
            auth,
            two_factor_auth,
        })
    }

    fn inject_session_cookies(&self, session: &StoredSession) {
        self.cookie_jar.add_cookie_str(
            &format!("auth={}; Path=/; Secure; HttpOnly", session.auth),
            &self.base_url,
        );

        if let Some(two_factor_auth) = &session.two_factor_auth {
            self.cookie_jar.add_cookie_str(
                &format!(
                    "twoFactorAuth={}; Path=/; Secure; HttpOnly",
                    two_factor_auth
                ),
                &self.base_url,
            );
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
}

#[derive(Debug)]
enum CurrentUserError {
    Http(StatusCode),
    Network(reqwest::Error),
    Decode(reqwest::Error),
    InvalidUrl,
}

fn build_basic_auth(username: &str, password: &str) -> String {
    let username = urlencoding::encode(username);
    let password = urlencoding::encode(password);

    let raw = format!("{}:{}", username, password);
    let encoded = STANDARD.encode(raw);

    format!("Basic {}", encoded)
}

fn detect_two_factor_methods(user: &User) -> Vec<TwoFactorMethod> {
    user.requires_two_factor_auth
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|method| match method.as_str() {
            "totp" => TwoFactorMethod::Totp,
            "emailOtp" => TwoFactorMethod::EmailOtp,
            "otp" => TwoFactorMethod::RecoveryCode,
            other => TwoFactorMethod::Unknown(other.to_string()),
        })
        .collect()
}

fn find_cookie(cookie_header: &str, name: &str) -> Option<String> {
    cookie_header.split(';').find_map(|cookie| {
        let cookie = cookie.trim();

        let (cookie_name, cookie_value) = cookie.split_once('=')?;

        if cookie_name == name {
            Some(cookie_value.to_string())
        } else {
            None
        }
    })
}
