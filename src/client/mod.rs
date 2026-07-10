pub mod config;
pub mod request;

use reqwest::Client;
use url::Url;

use crate::session::auth::Auth;

#[derive(Debug, Clone)]
pub struct VrcClient {
    pub config: config::ClientConfig,
    http: Client,
}

impl VrcClient {
    pub fn new(http: Client, config: config::ClientConfig) -> Self {
        Self { config, http }
    }

    pub fn from_auth(auth: &Auth) -> Self {
        Self::new(
            auth.client().clone(),
            config::ClientConfig::new(auth.base_url().clone(), auth.user_agent().to_string()),
        )
    }

    pub fn with_base_url(http: Client, base_url: Url, user_agent: impl Into<String>) -> Self {
        Self::new(http, config::ClientConfig::new(base_url, user_agent))
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    pub fn endpoint(&self, path: &str) -> Result<Url, url::ParseError> {
        self.config.base_url.join(path.trim_start_matches('/'))
    }
}
