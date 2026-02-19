#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub id: String,
    pub username: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "image")]
    pub avatar_url: Option<String>,
}