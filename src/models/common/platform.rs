#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Android,
    Ios,
    StandaloneWindows,
}
