#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpostorQueueStats {
    pub pending: u32,
    pub processing: u32,
    pub completed: u32,
    pub failed: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpostorStatus {
    pub avatar_id: String,
    pub status: String,
    pub queued_at: Option<String>,
    pub completed_at: Option<String>,
}
