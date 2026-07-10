use serde::Serialize;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        file::{
            File, FileAnalysis, FileCreate, FileDataUploadFinish, FileDataUploadStart,
            FileDataUploadStatus, FileList, FileVersionCreate,
        },
        response::ApiResponse,
    },
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_number: Option<u32>,
}

impl VrcClient {
    pub async fn files(&self, query: &PaginationQuery) -> Result<FileList, VrcError> {
        self.get_json_with_query("files", query).await
    }

    pub async fn create_file(&self, body: &FileCreate) -> Result<File, VrcError> {
        self.post_json("file", body).await
    }

    pub async fn file(&self, file_id: &str) -> Result<File, VrcError> {
        self.get_json(&format!("file/{file_id}")).await
    }

    pub async fn delete_file(&self, file_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("file/{file_id}")).await
    }

    pub async fn create_file_version(
        &self,
        file_id: &str,
        body: &FileVersionCreate,
    ) -> Result<File, VrcError> {
        self.post_json(&format!("file/{file_id}"), body).await
    }

    pub async fn delete_file_version(&self, file_id: &str, version: u32) -> Result<File, VrcError> {
        self.delete_json(&format!("file/{file_id}/{version}")).await
    }

    pub async fn start_file_upload(
        &self,
        file_id: &str,
        version: u32,
        file_type: &str,
        body: &FileDataUploadStart,
    ) -> Result<FileDataUploadStatus, VrcError> {
        self.put_json(&format!("file/{file_id}/{version}/{file_type}/start"), body)
            .await
    }

    pub async fn finish_file_upload(
        &self,
        file_id: &str,
        version: u32,
        file_type: &str,
        body: &FileDataUploadFinish,
    ) -> Result<File, VrcError> {
        self.put_json(
            &format!("file/{file_id}/{version}/{file_type}/finish"),
            body,
        )
        .await
    }

    pub async fn file_upload_status(
        &self,
        file_id: &str,
        version: u32,
        file_type: &str,
    ) -> Result<FileDataUploadStatus, VrcError> {
        self.get_json(&format!("file/{file_id}/{version}/{file_type}/status"))
            .await
    }

    pub async fn file_analysis(
        &self,
        file_id: &str,
        version: u32,
    ) -> Result<FileAnalysis, VrcError> {
        self.get_json(&format!("analysis/{file_id}/{version}"))
            .await
    }

    pub async fn file_security_analysis(
        &self,
        file_id: &str,
        version: u32,
    ) -> Result<FileAnalysis, VrcError> {
        self.get_json(&format!("analysis/{file_id}/{version}/security"))
            .await
    }

    pub async fn file_standard_analysis(
        &self,
        file_id: &str,
        version: u32,
    ) -> Result<FileAnalysis, VrcError> {
        self.get_json(&format!("analysis/{file_id}/{version}/standard"))
            .await
    }
}
