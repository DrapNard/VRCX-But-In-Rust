use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        notification::{
            AcknowledgeNotifications, ClearNotifications, NotificationList, NotificationReply,
            NotificationRespond, NotificationResponse, NotificationV2,
        },
        response::ApiResponse,
    },
};

impl VrcClient {
    pub async fn notifications(
        &self,
        query: &PaginationQuery,
    ) -> Result<NotificationList, VrcError> {
        self.get_json_with_query("notifications", query).await
    }

    pub async fn notification(&self, notification_id: &str) -> Result<NotificationV2, VrcError> {
        self.get_json(&format!("notifications/{notification_id}"))
            .await
    }

    pub async fn acknowledge_notifications(
        &self,
        body: &AcknowledgeNotifications,
    ) -> Result<ApiResponse, VrcError> {
        self.post_json("auth/user/notifications", body).await
    }

    pub async fn clear_notifications(
        &self,
        body: &ClearNotifications,
    ) -> Result<ApiResponse, VrcError> {
        self.put_json("auth/user/notifications/clear", body).await
    }

    pub async fn delete_notification(
        &self,
        notification_id: &str,
    ) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("notifications/{notification_id}"))
            .await
    }

    pub async fn accept_notification(
        &self,
        notification_id: &str,
    ) -> Result<NotificationResponse, VrcError> {
        self.put_json(
            &format!("auth/user/notifications/{notification_id}/accept"),
            &(),
        )
        .await
    }

    pub async fn hide_notification(
        &self,
        notification_id: &str,
    ) -> Result<NotificationResponse, VrcError> {
        self.put_json(
            &format!("auth/user/notifications/{notification_id}/hide"),
            &(),
        )
        .await
    }

    pub async fn see_notification(
        &self,
        notification_id: &str,
    ) -> Result<NotificationResponse, VrcError> {
        self.put_json(&format!("notifications/{notification_id}/see"), &())
            .await
    }

    pub async fn reply_notification(
        &self,
        notification_id: &str,
        body: &NotificationReply,
    ) -> Result<NotificationResponse, VrcError> {
        self.post_json(&format!("notifications/{notification_id}/reply"), body)
            .await
    }

    pub async fn respond_notification(
        &self,
        notification_id: &str,
        body: &NotificationRespond,
    ) -> Result<NotificationResponse, VrcError> {
        self.post_json(&format!("notifications/{notification_id}/respond"), body)
            .await
    }
}
