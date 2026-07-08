use super::FileStatus;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDataUploadStatus {
    pub file_id: String,
    pub version: u32,
    pub file_type: String,
    pub status: FileStatus,
    pub upload_id: Option<String>,
    pub parts: Vec<FileUploadPart>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadPart {
    pub part_number: u32,
    pub etag: Option<String>,
    pub status: FileStatus,
}
