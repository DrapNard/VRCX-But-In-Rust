#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemUpdate {
    pub is_archived: Option<bool>,
    pub is_seen: Option<bool>,
    pub user_attributes: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryConsume {
    pub inventory_item_id: String,
    pub quantity: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryEquip {
    pub inventory_item_id: String,
    pub slot: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryShareDirect {
    pub inventory_item_id: String,
    pub receiver_user_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySharePedestal {
    pub inventory_item_id: String,
    pub world_id: Option<String>,
    pub instance_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySpawn {
    pub inventory_item_id: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryUnequipSlot {
    pub slot: String,
}
