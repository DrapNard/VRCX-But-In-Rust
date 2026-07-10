use super::FavoriteType;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroups {
    pub groups: Vec<FavoriteGroup>,
}

impl<'de> serde::Deserialize<'de> for FavoriteGroups {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde::Deserialize::deserialize(deserializer)?;
        let groups = match value {
            serde_json::Value::Array(groups) => groups,
            serde_json::Value::Object(mut object) => object
                .remove("groups")
                .and_then(|groups| groups.as_array().cloned())
                .ok_or_else(|| serde::de::Error::custom("favorite groups payload is invalid"))?,
            _ => {
                return Err(serde::de::Error::custom(
                    "expected favorite groups array or object",
                ));
            }
        };
        serde_json::from_value(groups.into())
            .map(|groups| Self { groups })
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroup {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub owner_id: String,
    #[serde(rename = "type", alias = "favoriteType")]
    pub favorite_type: FavoriteType,
    pub visibility: String,
    pub tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_bare_favorite_group_array() {
        let groups: FavoriteGroups = serde_json::from_value(serde_json::json!([{
            "id": "fvgrp_1",
            "name": "group_0",
            "displayName": "Friends",
            "ownerId": "usr_1",
            "type": "friend",
            "visibility": "private",
            "tags": ["group_0"]
        }]))
        .unwrap();

        assert_eq!(groups.groups.len(), 1);
        assert_eq!(groups.groups[0].name, "group_0");
        assert!(matches!(
            groups.groups[0].favorite_type,
            FavoriteType::Friend
        ));
    }
}
