//! Section domain models and request payload helpers.

use serde::{Deserialize, Serialize};

/// Compact section reference used in task memberships and other contexts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub struct SectionReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Section display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl SectionReference {
    /// Human readable label.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Full section payload returned from Asana.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Section {
    /// Section identifier.
    pub gid: String,
    /// Display name (the text displayed as the section header).
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Parent project reference.
    #[serde(default)]
    pub project: Option<SectionProjectReference>,
    /// Deprecated field - use project instead.
    #[serde(default)]
    pub projects: Vec<SectionProjectReference>,
}

/// Compact project reference used within section payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub struct SectionProjectReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Project name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl SectionProjectReference {
    /// Human readable label.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Payload for creating a section in a project.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct SectionCreateData {
    /// Section name (required).
    pub name: String,
    /// Optional positioning parameter: insert before this section gid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_before: Option<String>,
    /// Optional positioning parameter: insert after this section gid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_after: Option<String>,
}

/// API envelope for section create requests.
#[derive(Debug, Clone, Serialize)]
pub struct SectionCreateRequest {
    /// Wrapped data payload.
    pub data: SectionCreateData,
}

/// Payload for adding a task to a section.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct AddTaskToSectionData {
    /// Task gid to add to the section.
    pub task: String,
    /// Optional: insert task before this task gid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_before: Option<String>,
    /// Optional: insert task after this task gid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_after: Option<String>,
}

/// API envelope for add task to section requests.
#[derive(Debug, Clone, Serialize)]
pub struct AddTaskToSectionRequest {
    /// Wrapped data payload.
    pub data: AddTaskToSectionData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_reference_label_uses_name() {
        let section = SectionReference {
            gid: "123".to_string(),
            name: Some("In Progress".to_string()),
            resource_type: Some("section".to_string()),
        };
        assert_eq!(section.label(), "In Progress");
    }

    #[test]
    fn section_reference_label_fallback_to_gid() {
        let section = SectionReference {
            gid: "456".to_string(),
            name: None,
            resource_type: Some("section".to_string()),
        };
        assert_eq!(section.label(), "456");
    }

    #[test]
    fn create_request_serializes_correctly() {
        let request = SectionCreateRequest {
            data: SectionCreateData {
                name: "New Section".to_string(),
                insert_before: None,
                insert_after: Some("789".to_string()),
            },
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"New Section\""));
        assert!(json.contains("\"insert_after\":\"789\""));
        assert!(!json.contains("insert_before"));
    }

    #[test]
    fn add_task_request_serializes_correctly() {
        let request = AddTaskToSectionRequest {
            data: AddTaskToSectionData {
                task: "task123".to_string(),
                insert_before: Some("task456".to_string()),
                insert_after: None,
            },
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"task\":\"task123\""));
        assert!(json.contains("\"insert_before\":\"task456\""));
        assert!(!json.contains("insert_after"));
    }
}
