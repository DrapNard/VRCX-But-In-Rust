use serde::Serialize;

use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        inventory::{
            Inventory, InventoryCollections, InventoryConsume, InventoryDrops, InventoryEquip,
            InventoryItem, InventoryItemUpdate, InventoryShareDirect, InventorySharePedestal,
            InventorySpawn, InventoryTemplate,
        },
        response::ApiResponse,
    },
};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryQuery {
    #[serde(flatten)]
    pub page: PaginationQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
}

impl VrcClient {
    pub async fn inventory(&self, query: &InventoryQuery) -> Result<Inventory, VrcError> {
        self.get_json_with_query("inventory", query).await
    }

    pub async fn inventory_item(&self, item_id: &str) -> Result<InventoryItem, VrcError> {
        self.get_json(&format!("inventory/{item_id}")).await
    }

    pub async fn update_inventory_item(
        &self,
        item_id: &str,
        body: &InventoryItemUpdate,
    ) -> Result<InventoryItem, VrcError> {
        self.put_json(&format!("inventory/{item_id}"), body).await
    }

    pub async fn delete_inventory_item(&self, item_id: &str) -> Result<ApiResponse, VrcError> {
        self.delete_json(&format!("inventory/{item_id}")).await
    }

    pub async fn inventory_collections(
        &self,
        query: &PaginationQuery,
    ) -> Result<InventoryCollections, VrcError> {
        self.get_json_with_query("inventory/collections", query)
            .await
    }

    pub async fn inventory_drops(
        &self,
        query: &PaginationQuery,
    ) -> Result<InventoryDrops, VrcError> {
        self.get_json_with_query("inventory/drops", query).await
    }

    pub async fn inventory_template(
        &self,
        template_id: &str,
    ) -> Result<InventoryTemplate, VrcError> {
        self.get_json(&format!("inventory/template/{template_id}"))
            .await
    }

    pub async fn consume_inventory(
        &self,
        item_id: &str,
        body: &InventoryConsume,
    ) -> Result<InventoryItem, VrcError> {
        self.post_json(&format!("inventory/{item_id}/consume"), body)
            .await
    }

    pub async fn equip_inventory(
        &self,
        item_id: &str,
        body: &InventoryEquip,
    ) -> Result<InventoryItem, VrcError> {
        self.post_json(&format!("inventory/{item_id}/equip"), body)
            .await
    }

    pub async fn share_inventory_direct(
        &self,
        body: &InventoryShareDirect,
    ) -> Result<InventoryItem, VrcError> {
        self.post_json("inventory/cloning/direct", body).await
    }

    pub async fn share_inventory_pedestal(
        &self,
        body: &InventorySharePedestal,
    ) -> Result<InventoryItem, VrcError> {
        self.post_json("inventory/cloning/pedestal", body).await
    }

    pub async fn spawn_inventory(&self, body: &InventorySpawn) -> Result<InventoryItem, VrcError> {
        self.post_json("inventory/spawn", body).await
    }
}
