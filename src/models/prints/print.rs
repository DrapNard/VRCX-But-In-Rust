#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintFiles {
    pub file_id: String,
    pub image: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Print {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub owner_id: String,
    pub world_id: String,
    pub world_name: String,
    pub note: String,
    pub files: PrintFiles,
    pub timestamp: String,
    pub created_at: String,
}

pub type UserPrints = crate::models::common::Paginated<Print>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintUpload {
    pub print: Print,
}
