use crate::{client::VrcClient, error::VrcError, models::users::User};

impl VrcClient {
    pub async fn current_user(&self) -> Result<User, VrcError> {
        self.get_json("auth/user").await
    }
}
