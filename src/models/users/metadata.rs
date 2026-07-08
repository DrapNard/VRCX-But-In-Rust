#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub date_joined: String,
    pub last_activity: String,
    pub last_login: String,
    pub last_mobile: String,
}
