use serde::Serialize;
use serde_json::Value;

use crate::{api::PaginationQuery, client::VrcClient, error::VrcError, models::instance::Instance};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseInstanceQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard_close: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
}

impl VrcClient {
    pub async fn recent_locations(&self, query: &PaginationQuery) -> Result<Vec<String>, VrcError> {
        self.get_json_with_query("instances/recent", query).await
    }

    pub async fn instance_by_short_name(&self, short_name: &str) -> Result<Instance, VrcError> {
        self.get_json(&format!("instances/s/{short_name}")).await
    }

    pub async fn instance(
        &self,
        world_id: &str,
        instance_id: &str,
    ) -> Result<Option<Instance>, VrcError> {
        self.get_json(&format!("instances/{world_id}:{instance_id}"))
            .await
    }

    pub async fn close_instance(
        &self,
        world_id: &str,
        instance_id: &str,
        query: &CloseInstanceQuery,
    ) -> Result<Instance, VrcError> {
        self.delete_json_with_query(&format!("instances/{world_id}:{instance_id}"), query)
            .await
    }

    pub async fn instance_short_name(
        &self,
        world_id: &str,
        instance_id: &str,
    ) -> Result<Value, VrcError> {
        self.get_json(&format!("instances/{world_id}:{instance_id}/shortName"))
            .await
    }
}
