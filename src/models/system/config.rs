use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub client_api_key: Option<String>,
    pub client_version_standalone: Option<String>,
    pub dev_app_version_standalone: Option<String>,
    pub raw: Value,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Value::deserialize(deserializer)?;
        Ok(Self {
            client_api_key: string_field(&raw, "clientApiKey"),
            client_version_standalone: string_field(&raw, "clientVersionStandalone"),
            dev_app_version_standalone: string_field(&raw, "devAppVersionStandalone"),
            raw,
        })
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoPush {
    pub id: String,
    pub title: String,
    pub body: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTime {
    pub utc: String,
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn captures_full_config_as_raw_data() {
        let config: Config = serde_json::from_str(
            r#"{"clientApiKey":"key","clientVersionStandalone":"1","unmodeled":true}"#,
        )
        .unwrap();

        assert_eq!(config.client_api_key.as_deref(), Some("key"));
        assert_eq!(config.raw["unmodeled"], true);
    }
}
