use crate::models::common::ImagePair;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InventoryItemType {
    Prop,
    Emoji,
    Sticker,
    Print,
    Unknown,
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
    pub animated: Option<bool>,
    pub animation_style: Option<String>,
    pub asset_bundle_id: Option<String>,
    pub file_id: Option<String>,
    pub image_url: Option<String>,
    pub inventory_items_to_instantiate: Vec<String>,
    pub mask_tag: Option<String>,
    pub prop_id: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryUserAttributes {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub trail_color: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub id: String,
    pub holder_id: String,
    pub template_id: String,
    pub item_type: InventoryItemType,
    pub item_type_label: String,
    pub name: String,
    pub image_url: String,
    pub collections: Vec<String>,
    pub flags: Vec<String>,
    pub metadata: InventoryItemMetadata,
    pub user_attributes: InventoryUserAttributes,
    pub quantifiable: bool,
    pub is_archived: bool,
    pub is_seen: bool,
    pub tags: Vec<String>,
    pub validate_user_attributes: bool,
    pub created_at: String,
    pub updated_at: String,
    pub template_created_at: String,
    pub template_updated_at: String,
}
