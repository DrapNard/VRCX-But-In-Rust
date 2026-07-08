pub type UserNotes = crate::models::common::Paginated<UserNote>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserNote {
    pub id: String,
    pub target_user_id: String,
    pub note: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetReviewNote {
    pub asset_id: String,
    pub notes: String,
    pub updated_at: String,
}
