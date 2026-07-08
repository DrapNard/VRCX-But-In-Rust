use super::FavoriteType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteLimits {
    pub limits: Vec<FavoriteLimit>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteLimit {
    pub favorite_type: FavoriteType,
    pub count: u32,
    pub max: u32,
}
