use serde::Serialize;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        favorite::{
            Favorite, FavoriteAdd, FavoriteGroup, FavoriteGroupUpdate, FavoriteGroups,
            FavoriteLimits, FavoriteList, FavoriteType,
        },
        response::ApiResponse,
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<FavoriteType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGroupsQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
}

impl VrcClient {
    pub async fn favorite_limits(&self) -> Result<FavoriteLimits, VrcError> {
        self.get_json("auth/user/favoritelimits").await
    }

    pub async fn favorite_groups(&self) -> Result<FavoriteGroups, VrcError> {
        self.favorite_groups_with_query(&FavoriteGroupsQuery::default())
            .await
    }

    pub async fn favorite_groups_with_query(
        &self,
        query: &FavoriteGroupsQuery,
    ) -> Result<FavoriteGroups, VrcError> {
        self.get_json_with_query("favorite/groups", query).await
    }

    pub async fn favorite_group(
        &self,
        favorite_type: FavoriteType,
        group_name: &str,
        user_id: &str,
    ) -> Result<FavoriteGroup, VrcError> {
        let favorite_type = documented_favorite_type(favorite_type)?;
        self.get_json(&favorite_group_path(favorite_type, group_name, user_id))
            .await
    }

    pub async fn favorites(&self, query: &FavoritesQuery) -> Result<FavoriteList, VrcError> {
        self.get_json_with_query("favorites", query).await
    }

    pub async fn add_favorite(&self, body: &FavoriteAdd) -> Result<Favorite, VrcError> {
        self.post_json("favorites", body).await
    }

    pub async fn remove_favorite(&self, favorite_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("favorites/{favorite_id}")).await
    }

    pub async fn update_favorite_group(
        &self,
        favorite_type: FavoriteType,
        group_name: &str,
        user_id: &str,
        body: &FavoriteGroupUpdate,
    ) -> Result<(), VrcError> {
        let favorite_type = documented_favorite_type(favorite_type)?;
        self.put_empty(
            &favorite_group_path(favorite_type, group_name, user_id),
            body,
        )
        .await
    }

    pub async fn clear_favorite_group(
        &self,
        favorite_type: FavoriteType,
        group_name: &str,
        user_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        let favorite_type = documented_favorite_type(favorite_type)?;
        self.delete_json(&favorite_group_path(favorite_type, group_name, user_id))
            .await
    }
}

fn documented_favorite_type(favorite_type: FavoriteType) -> Result<&'static str, VrcError> {
    favorite_type.as_api_str().ok_or_else(|| {
        VrcError::Decode("favorite groups only support avatar, friend, or world".to_string())
    })
}

fn favorite_group_path(favorite_type: &str, group_name: &str, user_id: &str) -> String {
    format!(
        "favorite/group/{}/{}/{}",
        urlencoding::encode(favorite_type),
        urlencoding::encode(group_name),
        urlencoding::encode(user_id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favorite_group_query_uses_documented_names() {
        let query = FavoriteGroupsQuery {
            page: PaginationQuery::new().limit(100).offset(20),
            user_id: Some("usr_target".to_string()),
            owner_id: Some("usr_owner".to_string()),
        };
        let encoded = serde_urlencoded::to_string(query).unwrap();
        assert!(encoded.contains("n=100"));
        assert!(encoded.contains("offset=20"));
        assert!(encoded.contains("userId=usr_target"));
        assert!(encoded.contains("ownerId=usr_owner"));
    }
}
