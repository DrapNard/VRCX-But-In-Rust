#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentWarnings{
    Adult,
    Combat,
    Featured,
    Gore,
    Horror,
    Other,
    Sex,
    Violence,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SystemFlags{
    Approved,
    CreatedRecently,
    Labs,
    UpdatedRecently,
    PublishedRecently,
    MonetizedWorld,
    PositiveFunToExplore,
    JamTags
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    pub content_warnings: Vec<ContentWarnings>,
    pub system_flags: Vec<SystemFlags>,
    pub author_tags: Vec<String>,
    pub jam_tags: Vec<String>,
    pub admin_tags: Vec<String>,
    pub feature_tags: Vec<String>,
    pub event_tags: Vec<String>,
    pub raw: Vec<String>,
}
