use serde::Serialize;

use crate::{
    client::VrcClient,
    error::VrcError,
    models::{
        friend::{Boop, BoopCreate, FriendList, FriendRequest, FriendStatus},
        response::ApiResponse,
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline: Option<bool>,
}

impl FriendsQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn offline(mut self, offline: bool) -> Self {
        self.offline = Some(offline);
        self
    }
}

impl VrcClient {
    pub async fn friends(&self, query: FriendsQuery) -> Result<FriendList, VrcError> {
        self.get_json_with_query("auth/user/friends", &query).await
    }

    pub async fn send_friend_request(&self, user_id: &str) -> Result<FriendRequest, VrcError> {
        self.post_json(&format!("user/{user_id}/friendRequest"), &())
            .await
    }

    pub async fn delete_friend_request(&self, user_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("user/{user_id}/friendRequest"))
            .await
    }

    pub async fn unfriend(&self, user_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("auth/user/friends/{user_id}"))
            .await
    }

    pub async fn friend_status(&self, user_id: &str) -> Result<FriendStatus, VrcError> {
        self.get_json(&format!("user/{user_id}/friendStatus")).await
    }

    pub async fn boop(&self, user_id: &str, body: &BoopCreate) -> Result<Boop, VrcError> {
        self.post_json(&format!("users/{user_id}/boop"), body).await
    }
}
