use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: u32,
    pub key: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "icon")]
    pub icon_url: String,
    pub points: u32,
    pub hidden: bool,
    pub unlocked: bool,
    #[serde(rename = "unlockedAt")]
    pub unlocked_at: chrono::DateTime<chrono::Utc>,
}

// "unlock" response returns a lighter achievement payload in the docs
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AchievementLite {
    pub key: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "icon")]
    pub icon_url: String,
    pub points: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AchievementUnlockResponse {
    pub success: bool,
    pub already_unlocked: bool,
    pub achievement: AchievementLite,
}
