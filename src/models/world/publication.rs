#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusRelease {
    Public,
    Private,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication{
    pub release_status: StatusRelease,
    pub featured: bool,
    pub publication_date: Option<String>,
    pub labs_publication_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
