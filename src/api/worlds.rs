use serde::Serialize;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{instance::Instance, world::World},
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSearchQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzzy: Option<bool>,
}

impl VrcClient {
    pub async fn search_worlds(&self, query: &WorldSearchQuery) -> Result<Vec<World>, VrcError> {
        self.get_json_with_query("worlds", query).await
    }

    pub async fn active_worlds(&self, query: &WorldSearchQuery) -> Result<Vec<World>, VrcError> {
        self.get_json_with_query("worlds/active", query).await
    }

    pub async fn favorite_worlds(&self, query: &WorldSearchQuery) -> Result<Vec<World>, VrcError> {
        self.get_json_with_query("worlds/favorites", query).await
    }

    pub async fn recent_worlds(&self, query: &WorldSearchQuery) -> Result<Vec<World>, VrcError> {
        self.get_json_with_query("worlds/recent", query).await
    }

    pub async fn world(&self, world_id: &str) -> Result<World, VrcError> {
        self.get_json(&format!("worlds/{world_id}")).await
    }

    pub async fn world_instance(
        &self,
        world_id: &str,
        instance_id: &str,
    ) -> Result<Instance, VrcError> {
        self.get_json(&format!("worlds/{world_id}/{instance_id}"))
            .await
    }
}
