#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomyAccount {
    pub id: String,
    pub owner_id: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TiliaStatus {
    pub account_status: String,
    pub kyc_status: Option<String>,
    pub tos_required: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TiliaTos {
    pub version: String,
    pub accepted: bool,
    pub accepted_at: Option<String>,
}
