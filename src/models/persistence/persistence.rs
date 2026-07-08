#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPersistenceStatus {
    pub exists: bool,
    pub user_id: String,
    pub world_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPersistence {
    pub user_id: String,
    pub world_id: String,
    pub data: serde_json::Value,
    pub updated_at: String,
}
