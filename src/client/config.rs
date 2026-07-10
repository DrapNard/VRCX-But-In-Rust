use url::Url;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: Url,
    pub user_agent: String,
}

impl ClientConfig {
    pub fn new(base_url: Url, user_agent: impl Into<String>) -> Self {
        Self {
            base_url,
            user_agent: user_agent.into(),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://api.vrchat.cloud/api/1/")
                .expect("default VRChat API URL must be valid"),
            user_agent: "vrcx-rs/0.1".to_string(),
        }
    }
}
