#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteLimits {
    pub default_max_favorite_groups: u32,
    pub default_max_favorites_per_group: u32,
    pub max_favorite_groups: FavoriteLimitValues,
    pub max_favorites_per_group: FavoriteLimitValues,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteLimitValues {
    pub avatar: u32,
    pub friend: u32,
    pub world: u32,
    #[serde(default)]
    pub vrc_plus_world: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_documented_favorite_limits_shape() {
        let limits: FavoriteLimits = serde_json::from_value(serde_json::json!({
            "defaultMaxFavoriteGroups": 4,
            "defaultMaxFavoritesPerGroup": 100,
            "maxFavoriteGroups": { "avatar": 4, "friend": 4, "world": 4, "vrcPlusWorld": 1 },
            "maxFavoritesPerGroup": { "avatar": 100, "friend": 100, "world": 100, "vrcPlusWorld": 100 }
        }))
        .unwrap();

        assert_eq!(limits.max_favorite_groups.friend, 4);
        assert_eq!(limits.max_favorites_per_group.vrc_plus_world, 100);
    }
}
