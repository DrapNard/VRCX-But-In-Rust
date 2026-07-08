#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub client_api_key: Option<String>,
    pub client_version_standalone: Option<String>,
    pub dev_app_version_standalone: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoPush {
    pub id: String,
    pub title: String,
    pub body: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTime {
    pub utc: String,
}
