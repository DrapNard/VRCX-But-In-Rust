#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBundle {
    pub id: String,
    pub tokens: u64,
    pub price: u64,
    pub currency: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamTransaction {
    pub id: String,
    pub user_id: String,
    pub steam_id: String,
    pub status: String,
    pub amount: u64,
    pub currency: String,
    pub created_at: String,
}
