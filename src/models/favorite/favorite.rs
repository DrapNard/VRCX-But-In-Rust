#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FavoriteType {
    Friend,
    World,
    Avatar,
    Prop,
    Group,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub id: String,
    pub favorite_id: String,
    #[serde(rename = "type")]
    pub favorite_type: FavoriteType,
    pub tags: Vec<String>,
}

pub type FavoriteList = crate::models::common::Paginated<Favorite>;
