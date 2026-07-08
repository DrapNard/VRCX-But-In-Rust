#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub asset_url: Option<String>,
    pub asset_url_object: serde_json::Value,
    pub active_asset_review_id: Option<String>,
    pub pending_upload: bool,
}
