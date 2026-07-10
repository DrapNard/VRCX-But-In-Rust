use crate::{
    client::VrcClient,
    error::VrcError,
    models::{
        permission::{AssignedPermission, Permission, PermissionList},
        response::ApiResponse,
        system::{
            Config, ContentAgreementStatus, ContentAgreementSubmit, Health, InfoPush, SystemTime,
        },
    },
};

impl VrcClient {
    pub async fn config(&self) -> Result<Config, VrcError> {
        self.get_json("config").await
    }

    pub async fn health(&self) -> Result<Health, VrcError> {
        self.get_json("health").await
    }

    pub async fn info_pushes(&self) -> Result<Vec<InfoPush>, VrcError> {
        self.get_json("infoPush").await
    }

    pub async fn system_time(&self) -> Result<SystemTime, VrcError> {
        self.get_json("time").await
    }

    pub async fn permissions(&self) -> Result<PermissionList, VrcError> {
        self.get_json("permissions").await
    }

    pub async fn assigned_permissions(&self) -> Result<Vec<AssignedPermission>, VrcError> {
        self.get_json("auth/permissions").await
    }

    pub async fn permission(&self, permission_id: &str) -> Result<Permission, VrcError> {
        self.get_json(&format!("permissions/{permission_id}")).await
    }

    pub async fn content_agreement(&self) -> Result<ContentAgreementStatus, VrcError> {
        self.get_json("agreement").await
    }

    pub async fn submit_content_agreement(
        &self,
        body: &ContentAgreementSubmit,
    ) -> Result<ApiResponse, VrcError> {
        self.post_json("agreement", body).await
    }
}
