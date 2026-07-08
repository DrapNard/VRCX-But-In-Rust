#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReleaseStatus {
    Public,
    Private,
    Hidden,
    All,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub release_status: ReleaseStatus,
    pub featured: bool,
    pub searchable: bool,
    pub lock: bool,
    pub created_at: String,
    pub updated_at: String,
    pub listing_date: Option<String>,
}
