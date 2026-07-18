use super::FavoriteType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteAdd {
    pub favorite_id: String,
    #[serde(rename = "type")]
    pub favorite_type: FavoriteType,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroupUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favorite_add_uses_the_documented_type_field() {
        let body = serde_json::to_value(FavoriteAdd {
            favorite_id: "usr_1".to_string(),
            favorite_type: FavoriteType::Friend,
            tags: vec!["group_0".to_string()],
        })
        .unwrap();

        assert_eq!(body["type"], "friend");
        assert!(body.get("favoriteType").is_none());
    }
}
