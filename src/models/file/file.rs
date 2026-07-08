#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileStatus {
    Waiting,
    Complete,
    Error,
    Deleted,
    Unknown,
}

pub type FileList = crate::models::Common::Paginated<File>;

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
    pub version: u32,
    pub status: FileStatus,
    pub file: FileUploadPart,
    pub delta: Option<FileUploadPart>,
    pub signature: Option<FileUploadPart>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadPart {
    pub category: String,
    pub file_name: String,
    pub size_in_bytes: u64,
    pub md5: Option<String>,
    pub url: Option<String>,
    pub upload_id: Option<String>,
}
