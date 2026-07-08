#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Styles {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub supplementary: Vec<String>,
}
