#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Region {
    Us,
    UsEast,
    Europe,
    Japan,
    Unknown,
}
