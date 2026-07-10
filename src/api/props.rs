use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        props::{Prop, PropCreate, PropPublish, PropPublishStatus, PropUpdate, Props},
        response::ApiResponse,
    },
};

impl VrcClient {
    pub async fn props(&self, query: &PaginationQuery) -> Result<Props, VrcError> {
        self.get_json_with_query("props", query).await
    }

    pub async fn create_prop(&self, body: &PropCreate) -> Result<Prop, VrcError> {
        self.post_json("props", body).await
    }

    pub async fn prop(&self, prop_id: &str) -> Result<Prop, VrcError> {
        self.get_json(&format!("props/{prop_id}")).await
    }

    pub async fn update_prop(&self, prop_id: &str, body: &PropUpdate) -> Result<Prop, VrcError> {
        self.put_json(&format!("props/{prop_id}"), body).await
    }

    pub async fn delete_prop(&self, prop_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("props/{prop_id}")).await
    }

    pub async fn prop_publish_status(&self, prop_id: &str) -> Result<PropPublishStatus, VrcError> {
        self.get_json(&format!("props/{prop_id}/publish")).await
    }

    pub async fn publish_prop(&self, prop_id: &str, body: &PropPublish) -> Result<Prop, VrcError> {
        self.put_json(&format!("props/{prop_id}/publish"), body)
            .await
    }
}
