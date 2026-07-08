use crate::models::common::platform::Platform;

use super::PerformanceRating;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityPackageUrl {
    pub unity_package_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityPackage {
    pub id: String,
    pub asset_url: String,
    pub asset_url_object: serde_json::Value,
    pub asset_version: u32,
    pub created_at: String,
    pub impostor_url: Option<String>,
    pub impostorizer_version: Option<String>,
    pub performance_rating: PerformanceRating,
    pub platform: Platform,
    pub plugin_url: Option<String>,
    pub plugin_url_object: serde_json::Value,
    pub scan_status: String,
    pub unity_sort_number: u64,
    pub unity_version: String,
    pub variant: String,
    pub world_signature: Option<String>,
}
