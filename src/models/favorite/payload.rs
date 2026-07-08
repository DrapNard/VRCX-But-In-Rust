use super::FavoriteType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteAdd {
    pub favorite_id: String,
    pub favorite_type: FavoriteType,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroupUpdate {
    pub display_name: Option<String>,
    pub visibility: Option<String>,
    pub tags: Vec<String>,
}
