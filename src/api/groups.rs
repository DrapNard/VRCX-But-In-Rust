use serde::Serialize;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        group::{
            Group, GroupAnnouncement, GroupAnnouncementCreate, GroupAuditLogTypes, GroupAuditLogs,
            GroupCreate, GroupGallery, GroupGalleryCreate, GroupGalleryImage, GroupGalleryUpdate,
            GroupInstance, GroupInviteCreate, GroupJoinRequestResponse, GroupMember,
            GroupMemberUpdate, GroupPermission, GroupPost, GroupPostCreate, GroupPostUpdate,
            GroupPosts, GroupRole, GroupRoleCreate, GroupRoleTemplates, GroupRoleUpdate,
            GroupSearchResult, GroupTransferRequest, GroupTransferability, GroupUpdate,
        },
        response::ApiResponse,
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSearchQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAuditQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_ids: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_types: Option<String>,
}

impl VrcClient {
    pub async fn search_groups(
        &self,
        query: &GroupSearchQuery,
    ) -> Result<Vec<GroupSearchResult>, VrcError> {
        self.get_json_with_query("groups", query).await
    }

    pub async fn create_group(&self, body: &GroupCreate) -> Result<Group, VrcError> {
        self.post_json("groups", body).await
    }

    pub async fn group(&self, group_id: &str) -> Result<Group, VrcError> {
        self.get_json(&format!("groups/{group_id}")).await
    }

    pub async fn update_group(
        &self,
        group_id: &str,
        body: &GroupUpdate,
    ) -> Result<Group, VrcError> {
        self.put_json(&format!("groups/{group_id}"), body).await
    }

    pub async fn delete_group(&self, group_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}")).await
    }

    pub async fn group_role_templates(&self) -> Result<GroupRoleTemplates, VrcError> {
        self.get_json("groups/roleTemplates").await
    }

    pub async fn group_announcement(
        &self,
        group_id: &str,
    ) -> Result<Option<GroupAnnouncement>, VrcError> {
        self.get_json(&format!("groups/{group_id}/announcement"))
            .await
    }

    pub async fn create_group_announcement(
        &self,
        group_id: &str,
        body: &GroupAnnouncementCreate,
    ) -> Result<GroupAnnouncement, VrcError> {
        self.post_json(&format!("groups/{group_id}/announcement"), body)
            .await
    }

    pub async fn delete_group_announcement(&self, group_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/announcement"))
            .await
    }

    pub async fn group_audit_log_types(
        &self,
        group_id: &str,
    ) -> Result<GroupAuditLogTypes, VrcError> {
        self.get_json(&format!("groups/{group_id}/auditLogTypes"))
            .await
    }

    pub async fn group_audit_logs(
        &self,
        group_id: &str,
        query: &GroupAuditQuery,
    ) -> Result<GroupAuditLogs, VrcError> {
        self.get_json_with_query(&format!("groups/{group_id}/auditLogs"), query)
            .await
    }

    pub async fn block_group(&self, group_id: &str) -> Result<ApiResponse, VrcError> {
        self.post_json(&format!("groups/{group_id}/block"), &())
            .await
    }

    pub async fn unblock_group(&self, group_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/block")).await
    }

    pub async fn group_galleries(&self, group_id: &str) -> Result<Vec<GroupGallery>, VrcError> {
        self.get_json(&format!("groups/{group_id}/galleries")).await
    }

    pub async fn create_group_gallery(
        &self,
        group_id: &str,
        body: &GroupGalleryCreate,
    ) -> Result<GroupGallery, VrcError> {
        self.post_json(&format!("groups/{group_id}/galleries"), body)
            .await
    }

    pub async fn update_group_gallery(
        &self,
        group_id: &str,
        gallery_id: &str,
        body: &GroupGalleryUpdate,
    ) -> Result<GroupGallery, VrcError> {
        self.put_json(&format!("groups/{group_id}/galleries/{gallery_id}"), body)
            .await
    }

    pub async fn delete_group_gallery(
        &self,
        group_id: &str,
        gallery_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/galleries/{gallery_id}"))
            .await
    }

    pub async fn group_gallery_images(
        &self,
        group_id: &str,
        gallery_id: &str,
        query: &PaginationQuery,
    ) -> Result<Vec<GroupGalleryImage>, VrcError> {
        self.get_json_with_query(
            &format!("groups/{group_id}/galleries/{gallery_id}/images"),
            query,
        )
        .await
    }

    pub async fn delete_group_gallery_image(
        &self,
        group_id: &str,
        gallery_id: &str,
        image_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!(
            "groups/{group_id}/galleries/{gallery_id}/images/{image_id}"
        ))
        .await
    }

    pub async fn group_instances(&self, group_id: &str) -> Result<Vec<GroupInstance>, VrcError> {
        self.get_json(&format!("groups/{group_id}/instances")).await
    }

    pub async fn group_invites(&self, group_id: &str) -> Result<Vec<GroupMember>, VrcError> {
        self.get_json(&format!("groups/{group_id}/invites")).await
    }

    pub async fn invite_to_group(
        &self,
        group_id: &str,
        body: &GroupInviteCreate,
    ) -> Result<GroupMember, VrcError> {
        self.post_json(&format!("groups/{group_id}/invites"), body)
            .await
    }

    pub async fn delete_group_invite(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/invites/{user_id}"))
            .await
    }

    pub async fn join_group(&self, group_id: &str) -> Result<GroupMember, VrcError> {
        self.post_json(&format!("groups/{group_id}/join"), &())
            .await
    }

    pub async fn leave_group(&self, group_id: &str) -> Result<ApiResponse, VrcError> {
        self.post_json(&format!("groups/{group_id}/leave"), &())
            .await
    }

    pub async fn group_members(
        &self,
        group_id: &str,
        query: &PaginationQuery,
    ) -> Result<Vec<GroupMember>, VrcError> {
        self.get_json_with_query(&format!("groups/{group_id}/members"), query)
            .await
    }

    pub async fn group_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<GroupMember, VrcError> {
        self.get_json(&format!("groups/{group_id}/members/{user_id}"))
            .await
    }

    pub async fn update_group_member(
        &self,
        group_id: &str,
        user_id: &str,
        body: &GroupMemberUpdate,
    ) -> Result<GroupMember, VrcError> {
        self.put_json(&format!("groups/{group_id}/members/{user_id}"), body)
            .await
    }

    pub async fn kick_group_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/members/{user_id}"))
            .await
    }

    pub async fn group_permissions(
        &self,
        group_id: &str,
    ) -> Result<Vec<GroupPermission>, VrcError> {
        self.get_json(&format!("groups/{group_id}/permissions"))
            .await
    }

    pub async fn group_posts(
        &self,
        group_id: &str,
        query: &PaginationQuery,
    ) -> Result<GroupPosts, VrcError> {
        self.get_json_with_query(&format!("groups/{group_id}/posts"), query)
            .await
    }

    pub async fn create_group_post(
        &self,
        group_id: &str,
        body: &GroupPostCreate,
    ) -> Result<GroupPost, VrcError> {
        self.post_json(&format!("groups/{group_id}/posts"), body)
            .await
    }

    pub async fn update_group_post(
        &self,
        group_id: &str,
        post_id: &str,
        body: &GroupPostUpdate,
    ) -> Result<GroupPost, VrcError> {
        self.put_json(&format!("groups/{group_id}/posts/{post_id}"), body)
            .await
    }

    pub async fn delete_group_post(
        &self,
        group_id: &str,
        post_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/posts/{post_id}"))
            .await
    }

    pub async fn group_join_requests(
        &self,
        group_id: &str,
        query: &PaginationQuery,
    ) -> Result<Vec<GroupMember>, VrcError> {
        self.get_json_with_query(&format!("groups/{group_id}/requests"), query)
            .await
    }

    pub async fn respond_group_join_request(
        &self,
        group_id: &str,
        user_id: &str,
        body: &GroupJoinRequestResponse,
    ) -> Result<GroupMember, VrcError> {
        self.put_json(&format!("groups/{group_id}/requests/{user_id}"), body)
            .await
    }

    pub async fn group_roles(&self, group_id: &str) -> Result<Vec<GroupRole>, VrcError> {
        self.get_json(&format!("groups/{group_id}/roles")).await
    }

    pub async fn create_group_role(
        &self,
        group_id: &str,
        body: &GroupRoleCreate,
    ) -> Result<GroupRole, VrcError> {
        self.post_json(&format!("groups/{group_id}/roles"), body)
            .await
    }

    pub async fn update_group_role(
        &self,
        group_id: &str,
        role_id: &str,
        body: &GroupRoleUpdate,
    ) -> Result<GroupRole, VrcError> {
        self.put_json(&format!("groups/{group_id}/roles/{role_id}"), body)
            .await
    }

    pub async fn delete_group_role(
        &self,
        group_id: &str,
        role_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("groups/{group_id}/roles/{role_id}"))
            .await
    }

    pub async fn transfer_group(
        &self,
        group_id: &str,
        body: &GroupTransferRequest,
    ) -> Result<GroupTransferability, VrcError> {
        self.post_json(&format!("groups/{group_id}/transfer"), body)
            .await
    }
}
