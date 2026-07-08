#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPurchaseCreate {
    pub listing_id: String,
    pub quantity: u32,
    pub target_user_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TiliaTosUpdate {
    pub accepted: bool,
    pub version: String,
}
