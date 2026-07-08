#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FavoriteType {
    Friend,
    World,
    Avatar,
    Prop,
    Group,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub id: String,
    pub favorite_id: String,
    pub favorite_type: FavoriteType,
    pub tags: Vec<String>,
}

pub type FavoriteList = crate::models::Common::Paginated<Favorite>;
