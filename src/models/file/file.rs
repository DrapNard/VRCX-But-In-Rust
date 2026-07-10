#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileStatus {
    Waiting,
    Complete,
    Error,
    Deleted,
    #[serde(other)]
    Unknown,
}

pub type FileList = crate::models::common::Paginated<File>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub mime_type: String,
    pub extension: String,
    pub tags: Vec<String>,
    pub versions: Vec<FileVersion>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVersion {
    #[serde(default)]
    pub version: u32,
    pub status: FileStatus,
    #[serde(default)]
    pub file: Option<FileUploadPart>,
    #[serde(default)]
    pub delta: Option<FileUploadPart>,
    #[serde(default)]
    pub signature: Option<FileUploadPart>,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadPart {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub size_in_bytes: u64,
    #[serde(default)]
    pub md5: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub upload_id: Option<String>,
}
