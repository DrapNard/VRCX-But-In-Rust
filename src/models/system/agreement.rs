#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAgreementStatus {
    pub accepted: bool,
    pub agreement_id: String,
    pub accepted_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAgreementSubmit {
    pub agreement_id: String,
    pub accepted: bool,
}
