#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Styles {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub supplementary: Vec<String>,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            primary: None,
            secondary: None,
            supplementary: Vec::new(),
        }
    }
}
