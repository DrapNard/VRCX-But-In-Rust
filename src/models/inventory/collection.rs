pub type InventoryCollections = crate::models::common::Paginated<InventoryCollection>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: String,
    pub item_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
