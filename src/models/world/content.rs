use crate::models::common::default_content_settings::DefaultContentSettings;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub default_content_settings: Vec<DefaultContentSettings>,
    pub unity_package: serde_json::Value,
    pub udon_products: serde_json::Value,
}
