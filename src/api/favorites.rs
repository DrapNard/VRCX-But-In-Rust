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

impl VrcClient {
    pub async fn favorite_limits(&self) -> Result<FavoriteLimits, VrcError> {
        self.get_json("auth/user/favoritelimits").await
    }

    pub async fn favorite_groups(&self) -> Result<FavoriteGroups, VrcError> {
        self.get_json("favorite/groups").await
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
        favorite_type: &str,
        group_name: &str,
        user_id: &str,
        body: &FavoriteGroupUpdate,
    ) -> Result<FavoriteGroup, VrcError> {
        self.put_json(
            &format!("favorite/group/{favorite_type}/{group_name}/{user_id}"),
            body,
        )
        .await
    }
}
