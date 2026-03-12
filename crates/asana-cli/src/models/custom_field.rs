//! Custom field metadata and helper types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Supported custom field value kinds surfaced by Asana.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomFieldType {
    /// Plain text.
    #[default]
    Text,
    /// Numeric value.
    Number,
    /// Single enum option.
    Enum,
    /// Multiple enum options.
    MultiEnum,
    /// Date (or date range).
    Date,
    /// People reference field.
    People,
    /// Percentage field.
    Percent,
    /// Currency amount field.
    Currency,
    /// Catch-all for newer types not explicitly handled.
    #[serde(other)]
    Unknown,
}

/// Enumeration option metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct CustomFieldEnumOption {
    /// Globally unique identifier.
    pub gid: String,
    /// Option display name.
    pub name: String,
    /// Optional colour slug.
    #[serde(default)]
    pub color: Option<String>,
    /// Whether the option is enabled.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

/// Date-based custom field payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct CustomFieldDateValue {
    /// Single date value (YYYY-MM-DD).
    #[serde(default)]
    pub date: Option<String>,
    /// Start date for ranges.
    #[serde(default)]
    pub start_on: Option<String>,
    /// Due date for ranges.
    #[serde(default)]
    pub due_on: Option<String>,
}

/// Fully hydrated custom field value record returned on tasks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct CustomField {
    /// Globally unique identifier.
    pub gid: String,
    /// Human readable name.
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Field type.
    #[serde(rename = "type")]
    pub field_type: CustomFieldType,
    /// Optional description/tooltip.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether the field is enabled.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Formatted value shown in the UI.
    #[serde(default)]
    pub display_value: Option<String>,
    /// Raw text value for text fields.
    #[serde(default)]
    pub text_value: Option<String>,
    /// Numeric value for number fields.
    #[serde(default)]
    pub number_value: Option<f64>,
    /// Percent value for percentage fields.
    #[serde(default)]
    pub percent_value: Option<f64>,
    /// ISO currency code when applicable.
    #[serde(default)]
    pub currency_code: Option<String>,
    /// Selected enum option.
    #[serde(default)]
    pub enum_value: Option<CustomFieldEnumOption>,
    /// Selected enum options for multi-select fields.
    #[serde(default)]
    pub multi_enum_values: Vec<CustomFieldEnumOption>,
    /// Date payload for date fields.
    #[serde(default)]
    pub date_value: Option<CustomFieldDateValue>,
    /// People references for people fields.
    #[serde(default)]
    pub people_value: Vec<String>,
    /// Additional metadata not explicitly modelled.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Input values accepted when creating or updating custom fields.
#[derive(Debug, Clone)]
pub enum CustomFieldValue {
    /// String-based value.
    Text(String),
    /// Floating point number.
    Number(f64),
    /// Boolean value.
    Bool(bool),
    /// Enum option gid.
    EnumOption(String),
    /// Multiple enum option gids.
    MultiEnum(Vec<String>),
    /// Date (YYYY-MM-DD).
    Date(String),
    /// Date range definition.
    DateRange {
        /// Start date (YYYY-MM-DD).
        start_on: Option<String>,
        /// Due date (YYYY-MM-DD).
        due_on: Option<String>,
    },
    /// Raw JSON payload for advanced scenarios.
    Json(Value),
}

impl CustomFieldValue {
    /// Convert the custom field value into a JSON representation that matches the API contract.
    #[must_use]
    pub fn into_value(self) -> Value {
        match self {
            Self::Text(value) => Value::String(value),
            Self::Number(value) => {
                serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
            }
            Self::Bool(value) => Value::Bool(value),
            Self::EnumOption(gid) => Value::String(gid),
            Self::MultiEnum(gids) => Value::Array(gids.into_iter().map(Value::String).collect()),
            Self::Date(date) => Value::String(date),
            Self::DateRange { start_on, due_on } => {
                let mut map = serde_json::Map::new();
                if let Some(start_on) = start_on {
                    map.insert("start_on".into(), Value::String(start_on));
                }
                if let Some(due_on) = due_on {
                    map.insert("due_on".into(), Value::String(due_on));
                }
                Value::Object(map)
            }
            Self::Json(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_text_value() {
        let value = CustomFieldValue::Text("hello".into()).into_value();
        assert_eq!(value, Value::String("hello".into()));
    }

    #[test]
    fn converts_multi_enum_value() {
        let value = CustomFieldValue::MultiEnum(vec!["111".into(), "222".into()]).into_value();
        assert_eq!(
            value,
            Value::Array(vec![
                Value::String("111".into()),
                Value::String("222".into())
            ])
        );
    }

    #[test]
    fn converts_date_range() {
        let value = CustomFieldValue::DateRange {
            start_on: Some("2024-01-01".into()),
            due_on: None,
        }
        .into_value();
        assert!(
            value
                .as_object()
                .and_then(|map| map.get("start_on"))
                .is_some()
        );
    }
}
