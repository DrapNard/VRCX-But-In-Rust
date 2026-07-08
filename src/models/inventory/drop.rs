use super::InventoryItemSummary;

pub type InventoryDrops = crate::models::common::Paginated<InventoryDrop>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDrop {
    pub id: String,
    pub author_id: String,
    pub receiver_id: String,
    pub inventory_item: InventoryItemSummary,
    pub message: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub redeemed_at: Option<String>,
}
