pub type ModerationReports = crate::models::Common::Paginated<ModerationReport>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationReport {
    pub id: String,
    pub author_id: String,
    pub target_id: String,
    pub target_type: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
