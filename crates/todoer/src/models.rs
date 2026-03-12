use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    New,
    InProgress,
    Completed,
    Abandoned,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::New => "NEW",
            Status::InProgress => "IN-PROGRESS",
            Status::Completed => "COMPLETED",
            Status::Abandoned => "ABANDONED",
        }
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NEW" => Ok(Status::New),
            "IN-PROGRESS" => Ok(Status::InProgress),
            "COMPLETED" => Ok(Status::Completed),
            "ABANDONED" => Ok(Status::Abandoned),
            _ => Err(format!("invalid status: {s}")),
        }
    }
}

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Status::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_key: String,
    pub created_at: String,
    pub description: String,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNote {
    pub id: i64,
    pub task_id: String,
    pub created_at: String,
    pub note: String,
}

pub fn normalize_project_key(name: &str) -> String {
    name.trim().to_lowercase()
}
