use serde::Serialize;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        avatar::{Avatar, Styles},
        impostor::{ImpostorQueueStats, ImpostorStatus},
        response::ApiResponse,
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarSearchQuery {
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
}

impl VrcClient {
    pub async fn avatar_styles(&self) -> Result<Styles, VrcError> {
        self.get_json("avatarStyles").await
    }

    pub async fn search_avatars(&self, query: &AvatarSearchQuery) -> Result<Vec<Avatar>, VrcError> {
        self.get_json_with_query("avatars", query).await
    }

    pub async fn favorite_avatars(
        &self,
        query: &AvatarSearchQuery,
    ) -> Result<Vec<Avatar>, VrcError> {
        self.get_json_with_query("avatars/favorites", query).await
    }

    pub async fn licensed_avatars(&self, query: &PaginationQuery) -> Result<Vec<Avatar>, VrcError> {
        self.get_json_with_query("avatars/licensed", query).await
    }

    pub async fn avatar(&self, avatar_id: &str) -> Result<Avatar, VrcError> {
        self.get_json(&format!("avatars/{avatar_id}")).await
    }

    pub async fn select_avatar(&self, avatar_id: &str) -> Result<Avatar, VrcError> {
        self.put_json(&format!("avatars/{avatar_id}/select"), &())
            .await
    }

    pub async fn select_fallback_avatar(&self, avatar_id: &str) -> Result<Avatar, VrcError> {
        self.put_json(&format!("avatars/{avatar_id}/selectFallback"), &())
            .await
    }

    pub async fn impostor_queue_stats(&self) -> Result<ImpostorQueueStats, VrcError> {
        self.get_json("avatars/impostor/queue/stats").await
    }

    pub async fn avatar_impostor(&self, avatar_id: &str) -> Result<ImpostorStatus, VrcError> {
        self.get_json(&format!("avatars/{avatar_id}/impostor"))
            .await
    }

    pub async fn enqueue_avatar_impostor(
        &self,
        avatar_id: &str,
    ) -> Result<ImpostorStatus, VrcError> {
        self.post_json(&format!("avatars/{avatar_id}/impostor/enqueue"), &())
            .await
    }

    pub async fn delete_avatar_impostor(&self, avatar_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("avatars/{avatar_id}/impostor"))
            .await
    }
}
