/// Represents a Gamplo player.
/// Used in responses from [`crate::Gamplo::get_player`], 
/// [`crate::Gamplo::from_token_with_player`], and [`crate::Gamplo::new_with_player`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct Player {
    pub id: String,
    pub username: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "image")]
    pub avatar_url: Option<String>,
}
