#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCreate {
    pub name: String,
    pub mime_type: String,
    pub extension: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionCreate {
    pub signature_md5: Option<String>,
    pub signature_size_in_bytes: Option<u64>,
    pub file_md5: String,
    pub file_size_in_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDataUploadStart {
    pub file_id: String,
    pub version: u32,
    pub file_type: String,
    pub part_number: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDataUploadFinish {
    pub file_id: String,
    pub version: u32,
    pub file_type: String,
    pub etags: Vec<String>,
}
