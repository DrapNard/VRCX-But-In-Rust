#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileAnalysisStatus {
    Waiting,
    Scanning,
    Complete,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAnalysis {
    pub file_id: String,
    pub version: u32,
    pub status: FileAnalysisStatus,
    pub standard: Option<FileAnalysisResult>,
    pub security: Option<FileAnalysisResult>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAnalysisResult {
    pub status: FileAnalysisStatus,
    pub result: serde_json::Value,
}
