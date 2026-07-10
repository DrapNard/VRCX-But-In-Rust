use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupAdminTag {
    AgeVerificationEnabled,
    HideMemberCount,
    FeaturedEventsEnabled,
    VrcEventGroupFairEnabled,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupTags {
    pub admin_tags: Vec<GroupAdminTag>,
    pub raw: Vec<String>,
}

impl<'de> Deserialize<'de> for GroupTags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let raw = match value {
            Value::Array(tags) => tags,
            Value::Object(mut object) => object
                .remove("raw")
                .and_then(|value| value.as_array().cloned())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
        .into_iter()
        .filter_map(|tag| tag.as_str().map(str::to_string))
        .collect();

        Ok(Self {
            admin_tags: Vec::new(),
            raw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::GroupTags;

    #[test]
    fn decodes_flat_group_tags() {
        let tags: GroupTags =
            serde_json::from_str(r#"["admin_hide_member_count","language_eng"]"#).unwrap();

        assert_eq!(tags.raw.len(), 2);
    }
}
