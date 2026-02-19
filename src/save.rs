use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct SaveData {
    pub slot: u32,
    pub data: Value,
    /// API: "sizeBytes"
    pub size_bytes: u64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Saves {
    pub saves: Vec<SaveMetadata>,
    pub max_slots: u32,
    /// API: "maxSizeBytes"
    pub max_size_bytes: u64,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct SaveMetadata {
    pub slot: u32,
    /// API: "sizeBytes"
    pub size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Responses for write/delete save endpoints
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct SaveWriteResponse {
    pub success: bool,
    pub slot: u32,
    pub size_bytes: u64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct SaveDeleteResponse {
    pub success: bool,
    pub deleted: bool,
}
