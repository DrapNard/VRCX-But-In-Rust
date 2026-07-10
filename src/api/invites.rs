use crate::{
    client::VrcClient,
    error::VrcError,
    models::{
        invite::{
            InviteMessage, InviteMessageList, InviteMessageSlot, InviteMessageType,
            InvitePhotoResponse, InviteWithPhoto,
        },
        notification::NotificationResponse,
    },
};

fn message_type(value: &InviteMessageType) -> &'static str {
    match value {
        InviteMessageType::Message => "message",
        InviteMessageType::Response => "response",
        InviteMessageType::Request => "request",
        InviteMessageType::RequestResponse => "requestResponse",
        InviteMessageType::Unknown => "unknown",
    }
}

impl VrcClient {
    pub async fn invite_myself(
        &self,
        world_id: &str,
        instance_id: &str,
    ) -> Result<NotificationResponse, VrcError> {
        self.post_json(&format!("invite/myself/to/{world_id}:{instance_id}"), &())
            .await
    }

    pub async fn invite_user(
        &self,
        user_id: &str,
        body: &InviteWithPhoto,
    ) -> Result<NotificationResponse, VrcError> {
        self.post_json(&format!("invite/{user_id}"), body).await
    }

    pub async fn respond_invite(
        &self,
        notification_id: &str,
        body: &InvitePhotoResponse,
    ) -> Result<NotificationResponse, VrcError> {
        self.post_json(&format!("invite/{notification_id}/response"), body)
            .await
    }

    pub async fn request_invite(
        &self,
        user_id: &str,
        body: &InviteWithPhoto,
    ) -> Result<NotificationResponse, VrcError> {
        self.post_json(&format!("requestInvite/{user_id}"), body)
            .await
    }

    pub async fn invite_messages(
        &self,
        user_id: &str,
        kind: &InviteMessageType,
    ) -> Result<InviteMessageList, VrcError> {
        self.get_json(&format!("message/{user_id}/{}", message_type(kind)))
            .await
    }

    pub async fn invite_message(
        &self,
        user_id: &str,
        kind: &InviteMessageType,
        slot: u8,
    ) -> Result<InviteMessage, VrcError> {
        self.get_json(&format!("message/{user_id}/{}/{slot}", message_type(kind)))
            .await
    }

    pub async fn update_invite_message(
        &self,
        user_id: &str,
        kind: &InviteMessageType,
        slot: u8,
        body: &InviteMessageSlot,
    ) -> Result<InviteMessageList, VrcError> {
        self.put_json(
            &format!("message/{user_id}/{}/{slot}", message_type(kind)),
            body,
        )
        .await
    }
}
