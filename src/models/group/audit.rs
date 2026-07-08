pub type GroupAuditLogTypes = Vec<String>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAuditLogs {
    pub has_next: bool,
    pub results: Vec<GroupAuditLog>,
    pub total_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAuditLog {
    pub id: String,
    pub group_id: String,
    pub actor_id: String,
    pub actor_display_name: String,
    pub target_id: Option<String>,
    pub event_type: String,
    pub description: String,
    pub data: serde_json::Value,
    pub created_at: String,
}
