use super::FavoriteType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroups {
    pub groups: Vec<FavoriteGroup>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroup {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub owner_id: String,
    pub favorite_type: FavoriteType,
    pub visibility: String,
    pub tags: Vec<String>,
}
