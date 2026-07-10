use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        moderation::{
            GlobalAvatarModeration, GlobalAvatarModerationCreate, ModerationReport,
            ModerationReportCreate, ModerationReports, PlayerModeration, PlayerModerationCreate,
        },
        response::ApiResponse,
    },
};

impl VrcClient {
    pub async fn player_moderations(&self) -> Result<Vec<PlayerModeration>, VrcError> {
        self.get_json("auth/user/playermoderations").await
    }

    pub async fn moderate_player(
        &self,
        body: &PlayerModerationCreate,
    ) -> Result<PlayerModeration, VrcError> {
        self.post_json("auth/user/playermoderations", body).await
    }

    pub async fn unmoderate_player(
        &self,
        body: &PlayerModerationCreate,
    ) -> Result<ApiResponse, VrcError> {
        self.put_json("auth/user/unplayermoderate", body).await
    }

    pub async fn avatar_moderations(&self) -> Result<Vec<GlobalAvatarModeration>, VrcError> {
        self.get_json("auth/user/avatarmoderations").await
    }

    pub async fn moderate_avatar(
        &self,
        body: &GlobalAvatarModerationCreate,
    ) -> Result<GlobalAvatarModeration, VrcError> {
        self.post_json("auth/user/avatarmoderations", body).await
    }

    pub async fn moderation_reports(
        &self,
        query: &PaginationQuery,
    ) -> Result<ModerationReports, VrcError> {
        self.get_json_with_query("moderationReports", query).await
    }

    pub async fn create_moderation_report(
        &self,
        body: &ModerationReportCreate,
    ) -> Result<ModerationReport, VrcError> {
        self.post_json("moderationReports", body).await
    }

    pub async fn moderation_report(&self, report_id: &str) -> Result<ModerationReport, VrcError> {
        self.get_json(&format!("moderationReports/{report_id}"))
            .await
    }
}
