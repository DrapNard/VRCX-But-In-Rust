use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        prints::{Print, PrintEdit, PrintUpload, UserPrints},
        response::ApiResponse,
    },
};

impl VrcClient {
    pub async fn upload_print(&self, body: &PrintUpload) -> Result<Print, VrcError> {
        self.post_json("prints", body).await
    }

    pub async fn user_prints(
        &self,
        user_id: &str,
        query: &PaginationQuery,
    ) -> Result<UserPrints, VrcError> {
        self.get_json_with_query(&format!("prints/user/{user_id}"), query)
            .await
    }

    pub async fn print(&self, print_id: &str) -> Result<Print, VrcError> {
        self.get_json(&format!("prints/{print_id}")).await
    }

    pub async fn edit_print(&self, print_id: &str, body: &PrintEdit) -> Result<Print, VrcError> {
        self.put_json(&format!("prints/{print_id}"), body).await
    }

    pub async fn delete_print(&self, print_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("prints/{print_id}")).await
    }
}
