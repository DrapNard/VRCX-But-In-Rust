use super::{InventoryItemMetadata, InventoryItemType, InventoryUserAttributes};

pub type InventoryTemplates = crate::models::common::Paginated<InventoryTemplate>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTemplate {
    pub id: String,
    pub item_type: InventoryItemType,
    pub item_type_label: String,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub metadata: InventoryItemMetadata,
    pub user_attributes: InventoryUserAttributes,
    pub quantifiable: bool,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
