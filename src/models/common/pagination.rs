use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_count: u32,
}

impl<'de, T> Deserialize<'de> for Paginated<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let (items_value, explicit_total) = match value {
            Value::Array(items) => (Value::Array(items), None),
            Value::Object(mut object) => {
                let total = object
                    .remove("totalCount")
                    .or_else(|| object.remove("total"))
                    .and_then(|value| value.as_u64())
                    .map(|value| value as u32);
                let items = object
                    .remove("items")
                    .or_else(|| object.remove("data"))
                    .or_else(|| object.remove("results"))
                    .ok_or_else(|| {
                        serde::de::Error::custom(
                            "paginated object is missing items, data, or results",
                        )
                    })?;
                (items, total)
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "expected a paginated object or an array",
                ));
            }
        };
        let items =
            serde_json::from_value::<Vec<T>>(items_value).map_err(serde::de::Error::custom)?;
        let total_count = explicit_total.unwrap_or(items.len() as u32);

        Ok(Self { items, total_count })
    }
}

#[cfg(test)]
mod tests {
    use super::Paginated;

    #[test]
    fn decodes_bare_array() {
        let page: Paginated<u32> = serde_json::from_str("[1,2,3]").unwrap();

        assert_eq!(page.items, vec![1, 2, 3]);
        assert_eq!(page.total_count, 3);
    }

    #[test]
    fn decodes_empty_bare_array() {
        let page: Paginated<u32> = serde_json::from_str("[]").unwrap();

        assert!(page.items.is_empty());
        assert_eq!(page.total_count, 0);
    }

    #[test]
    fn decodes_paginated_object() {
        let page: Paginated<u32> =
            serde_json::from_str(r#"{"items":[1,2],"totalCount":12}"#).unwrap();

        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.total_count, 12);
    }

    #[test]
    fn decodes_inventory_data_wrapper() {
        let page: Paginated<u32> =
            serde_json::from_str(r#"{"data":[1,2],"totalCount":7}"#).unwrap();

        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.total_count, 7);
    }
}
