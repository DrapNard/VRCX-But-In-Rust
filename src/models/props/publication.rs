#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropReleaseStatus {
    Public,
    Private,
    Hidden,
    Unknown,
}
