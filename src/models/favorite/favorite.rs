#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

impl FavoriteType {
    pub const fn as_api_str(self) -> Option<&'static str> {
        match self {
            Self::Friend => Some("friend"),
            Self::World => Some("world"),
            Self::Avatar => Some("avatar"),
            Self::Prop | Self::Group | Self::Unknown => None,
        }
    }
}

pub type FavoriteList = crate::models::common::Paginated<Favorite>;
