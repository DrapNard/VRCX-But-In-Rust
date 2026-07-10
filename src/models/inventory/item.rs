use crate::models::common::ImagePair;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InventoryItemType {
    Prop,
    Emoji,
    Sticker,
    Print,
    #[serde(other)]
    Unknown,
}

impl Default for InventoryItemType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemSummary {
    pub id: String,
    pub item_type: InventoryItemType,
    pub name: String,
    pub media: ImagePair,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemMetadata {
    #[serde(default)]
    pub animated: Option<bool>,
    #[serde(default)]
    pub animation_style: Option<String>,
    #[serde(default)]
    pub asset_bundle_id: Option<String>,
    #[serde(default)]
    pub file_id: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub inventory_items_to_instantiate: Vec<String>,
    #[serde(default)]
    pub mask_tag: Option<String>,
    #[serde(default)]
    pub prop_id: Option<String>,
    #[serde(default)]
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryUserAttributes {
    #[serde(default)]
    pub primary_color: Option<String>,
    #[serde(default)]
    pub secondary_color: Option<String>,
    #[serde(default)]
    pub trail_color: Option<String>,
    #[serde(default)]
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub holder_id: String,
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub item_type: InventoryItemType,
    #[serde(default)]
    pub item_type_label: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub collections: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
    pub metadata: InventoryItemMetadata,
    pub user_attributes: InventoryUserAttributes,
    #[serde(default)]
    pub quantifiable: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub is_seen: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub validate_user_attributes: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub template_created_at: String,
    #[serde(default)]
    pub template_updated_at: String,
}
