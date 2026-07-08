use crate::models::common::platform::Platform;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropUnityPackage {
    pub asset_url: String,
    pub asset_version: u32,
    pub platform: Platform,
    pub prop_signature: String,
    pub unity_version: String,
    pub variant: String,
}
