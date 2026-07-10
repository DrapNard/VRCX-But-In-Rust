use serde::Serialize;
use serde_json::Value;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        group::{BlockedGroups, GroupInstance, GroupSummary, InvitedGroups},
        note::{UserNote, UserNoteUpdate, UserNotes},
        persistence::{UserPersistence, UserPersistenceStatus},
        users::{OnlineUsers, User, UserFeedback},
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSearchQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

impl VrcClient {
    pub async fn search_users(&self, query: &UserSearchQuery) -> Result<Vec<User>, VrcError> {
        self.get_json_with_query("users", query).await
    }

    pub async fn active_users(&self) -> Result<OnlineUsers, VrcError> {
        self.get_json("users/active").await
    }

    pub async fn user(&self, user_id: &str) -> Result<User, VrcError> {
        self.get_json(&format!("users/{user_id}")).await
    }

    pub async fn user_by_name(&self, username: &str) -> Result<User, VrcError> {
        self.get_json(&format!("users/{username}/name")).await
    }

    pub async fn user_feedback(&self, user_id: &str) -> Result<UserFeedback, VrcError> {
        self.get_json(&format!("users/{user_id}/feedback")).await
    }

    pub async fn user_groups(&self, user_id: &str) -> Result<Vec<GroupSummary>, VrcError> {
        self.get_json(&format!("users/{user_id}/groups")).await
    }

    pub async fn user_groups_raw(&self, user_id: &str) -> Result<Vec<Value>, VrcError> {
        self.get_json(&format!("users/{user_id}/groups")).await
    }

    pub async fn mutual_friends(&self, user_id: &str) -> Result<Vec<Value>, VrcError> {
        self.get_json_with_query(
            &format!("users/{user_id}/mutuals/friends"),
            &PaginationQuery::new().limit(100),
        )
        .await
    }

    pub async fn mutual_groups(&self, user_id: &str) -> Result<Vec<Value>, VrcError> {
        self.get_json_with_query(
            &format!("users/{user_id}/mutuals/groups"),
            &PaginationQuery::new().limit(100),
        )
        .await
    }

    pub async fn invited_groups(&self, user_id: &str) -> Result<InvitedGroups, VrcError> {
        self.get_json(&format!("users/{user_id}/groups/invited"))
            .await
    }

    pub async fn blocked_groups(&self, user_id: &str) -> Result<BlockedGroups, VrcError> {
        self.get_json(&format!("users/{user_id}/groups/userblocked"))
            .await
    }

    pub async fn user_group_instances(
        &self,
        user_id: &str,
    ) -> Result<Vec<GroupInstance>, VrcError> {
        self.get_json(&format!("users/{user_id}/instances/groups"))
            .await
    }

    pub async fn user_notes(&self, query: &PaginationQuery) -> Result<UserNotes, VrcError> {
        self.get_json_with_query("userNotes", query).await
    }

    pub async fn update_user_note(
        &self,
        note_id: &str,
        body: &UserNoteUpdate,
    ) -> Result<UserNote, VrcError> {
        self.post_json(&format!("userNotes/{note_id}"), body).await
    }

    pub async fn delete_user_note(&self, note_id: &str) -> Result<UserNote, VrcError> {
        self.delete_json(&format!("userNotes/{note_id}")).await
    }

    pub async fn user_persistence(&self, user_id: &str) -> Result<UserPersistence, VrcError> {
        self.get_json(&format!("users/{user_id}/persist")).await
    }

    pub async fn user_world_persistence(
        &self,
        user_id: &str,
        world_id: &str,
    ) -> Result<UserPersistence, VrcError> {
        self.get_json(&format!("users/{user_id}/{world_id}/persist"))
            .await
    }

    pub async fn user_world_persistence_status(
        &self,
        user_id: &str,
        world_id: &str,
    ) -> Result<UserPersistenceStatus, VrcError> {
        self.get_json(&format!("users/{user_id}/{world_id}/persist/exists"))
            .await
    }
}
