use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::jam::{Jam, JamSubmission, JamSubmissions, Jams},
};

impl VrcClient {
    pub async fn jams(&self, query: &PaginationQuery) -> Result<Jams, VrcError> {
        self.get_json_with_query("jams", query).await
    }

    pub async fn jam(&self, jam_id: &str) -> Result<Jam, VrcError> {
        self.get_json(&format!("jams/{jam_id}")).await
    }

    pub async fn jam_submissions(
        &self,
        jam_id: &str,
        query: &PaginationQuery,
    ) -> Result<JamSubmissions, VrcError> {
        self.get_json_with_query(&format!("jams/{jam_id}/submissions"), query)
            .await
    }

    pub async fn jam_submission(
        &self,
        jam_id: &str,
        submission_id: &str,
    ) -> Result<JamSubmission, VrcError> {
        self.get_json(&format!("jams/{jam_id}/submissions/{submission_id}"))
            .await
    }
}
