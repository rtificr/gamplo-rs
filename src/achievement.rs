use serde::{Deserialize, Serialize};

/// Represents a Gamplo achievement.
/// Used in responses from [`crate::Gamplo::get_achievements`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub(crate) id: u32,
    pub(crate) key: String,
    pub(crate) title: String,
    pub(crate) description: String,
    #[serde(rename = "icon")]
    pub(crate) icon_url: String,
    pub(crate) points: u32,
    pub(crate) hidden: bool,
    pub(crate) unlocked: bool,
    #[serde(rename = "unlockedAt")]
    pub(crate) unlocked_at: chrono::DateTime<chrono::Utc>,
}

impl Achievement {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn icon_url(&self) -> &str {
        &self.icon_url
    }
    pub fn points(&self) -> u32 {
        self.points
    }
    pub fn hidden(&self) -> bool {
        self.hidden
    }
    pub fn unlocked(&self) -> bool {
        self.unlocked
    }
    pub fn unlocked_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.unlocked_at
    }
}

/// A lighter version of [`Achievement`] used in the unlock response to avoid redundant fields.
/// Used in responses from [`crate::Gamplo::unlock_achievement`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AchievementLite {
    pub(crate) key: String,
    pub(crate) title: String,
    pub(crate) description: String,
    #[serde(rename = "icon")]
    pub(crate) icon_url: String,
    pub(crate) points: u32,
}
impl AchievementLite {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn icon_url(&self) -> &str {
        &self.icon_url
    }
    pub fn points(&self) -> u32 {
        self.points
    }
}
/// Response from unlocking an achievement with
/// [`crate::Gamplo::unlock_achievement`] and
/// [`crate::Gamplo::unlock_achievement_with_secret`].
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AchievementUnlockResponse {
    /// Whether the message was successfully processed. Note that this can be true even if the achievement was already unlocked.
    pub success: bool,
    /// Whether the achievement was already unlocked before this request. If true, the achievement was not unlocked again, but the response is still successful.
    pub already_unlocked: bool,
    /// The achievement that was attempted to be unlocked. Note that if `already_unlocked` is true, this achievement was not unlocked again, but is included for convenience.
    pub achievement: AchievementLite,
}
impl AchievementUnlockResponse {
    pub fn success(&self) -> bool {
        self.success
    }
    pub fn already_unlocked(&self) -> bool {
        self.already_unlocked
    }
    pub fn achievement(&self) -> &AchievementLite {
        &self.achievement
    }
}
