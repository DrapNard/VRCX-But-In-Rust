#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub message: Option<String>,
    pub status_code: u16,
}

impl<'de> serde::Deserialize<'de> for ApiResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        if let Some(success) = value.get_mut("success") {
            value = success.take();
        }
        #[derive(serde::Deserialize)]
        struct ResponseFields {
            message: Option<String>,
            #[serde(alias = "statusCode")]
            status_code: u16,
        }
        let fields: ResponseFields =
            serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            message: fields.message,
            status_code: fields.status_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ApiResponse;

    #[test]
    fn decodes_wrapped_success_response() {
        let response: ApiResponse = serde_json::from_str(
            r#"{"success":{"message":"favorite deleted!","status_code":200}}"#,
        )
        .unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[test]
    fn keeps_support_for_direct_camel_case_response() {
        let response: ApiResponse =
            serde_json::from_str(r#"{"message":"ok","statusCode":200}"#).unwrap();
        assert_eq!(response.status_code, 200);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptyResponse {
    pub ok: bool,
}
