//! Gamplo SDK for Rust
//! 
//! Provides a Rust interface for the Gamplo API.
//! Based on [Gamplo's JavaScript SDK](https://gamplo.com/developer/sdk) and is designed to be used in both server-side and client-side (WASM) Gamplo games.
//! For more information/examples, [read the SDK documentation](https://gamplo.com/developer/sdk)
//! 
//! # Features
//! - `client`: Enables client-side (WASM) functionality
//! - `server`: Enables server-side functionality

#[cfg(all(feature = "client", feature = "server"))]
compile_error!("feature \"client\" and feature \"server\" cannot be enabled at the same time");
#[cfg(not(any(feature = "client", feature = "server")))]
compile_error!("either feature \"client\" or feature \"server\" must be enabled");

pub mod achievement;
pub mod error;
pub mod player;
pub mod save;
pub mod util;

use error::GamploError;
use serde_json::json;
use web_sys::{js_sys::Reflect, wasm_bindgen::JsValue};

use crate::{
    achievement::{Achievement, AchievementUnlockResponse},
    player::Player,
    save::{SaveData, SaveWriteResponse, Saves},
    util::get_error,
};

/// The URL for gamplo.com.
pub const GAMPLO_URL: &str = "https://gamplo.com";

fn evaluate_url_path(path: &str) -> String {
    format!("{}{}", GAMPLO_URL, path)
}

/// Main Gamplo client struct for interacting with the Gamplo API.
#[derive(Debug, Clone)]
pub struct Gamplo {
    session_id: String,
    client: reqwest::Client,
}
impl Gamplo {
    /// Creates a new Gamplo client from an authentication token.
    pub async fn from_token(token: String) -> Result<Self, GamploError> {
        Ok(Self::from_token_with_player(token).await?.0)
    }
    /// Creates a new Gamplo client from an authentication token, and also returns the authenticated player if available.
    pub async fn from_token_with_player(
        token: String,
    ) -> Result<(Self, Option<Player>), GamploError> {
        let client = reqwest::Client::new();
        let text = client
            .post(evaluate_url_path("/api/sdk/auth"))
            .header("Content-Type", "application/json")
            .body(json!({ "token": token }).to_string())
            .send()
            .await?
            .text()
            .await?;

        if let Some(error) =
            get_error(
                &serde_json::from_str(&text).map_err(|e| GamploError::Deserialization {
                    type_name: "auth error response".to_string(),
                    data: text.clone(),
                    source: e,
                })?,
            )
        {
            return Err(GamploError::Authentication(error));
        }

        #[derive(serde::Deserialize)]
        struct AuthResponse {
            #[serde(rename = "sessionId")]
            session_id: String,
            player: Option<Player>,
        }
        let parsed: AuthResponse =
            serde_json::from_str(&text).map_err(|e| GamploError::Deserialization {
                type_name: "AuthResponse".to_string(),
                data: text.clone(),
                source: e,
            })?;

        let client_struct = Gamplo {
            session_id: parsed.session_id,
            client,
        };
        Ok((client_struct, parsed.player))
    }
    /// Creates a new Gamplo client using an auto-detected token.
    pub async fn new() -> Result<Self, GamploError> {
        let token = get_token()?;
        Self::from_token(token).await
    }
    /// Creates a new Gamplo client using an auto-detected token and also returns the authenticated player if available.
    pub async fn new_with_player() -> Result<(Self, Option<Player>), GamploError> {
        let token = get_token()?;
        Self::from_token_with_player(token).await
    }
    /// Gets the authenticated player for this client, if available.
    pub async fn get_player(&self) -> Result<Option<Player>, GamploError> {
        let value = self
            .client
            .get(evaluate_url_path("/api/sdk/player"))
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?
            .parse::<serde_json::Value>()?;

        let player_value = value
            .get("player")
            .ok_or_else(|| GamploError::MissingField {
                field: "player".to_string(),
                response: format!("{:?}", value),
            })?;

        if player_value.is_null() {
            return Ok(None);
        }

        let player: Player = serde_json::from_value(player_value.clone()).map_err(|err| {
            GamploError::Deserialization {
                type_name: "Player".to_string(),
                data: format!("{:?}", player_value),
                source: err,
            }
        })?;

        Ok(Some(player))
    }
    /// Gets all achievements for this client.
    pub async fn get_achievements(&self) -> Result<Vec<Achievement>, GamploError> {
        let value = self
            .client
            .get(evaluate_url_path("/api/sdk/achievements"))
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?
            .parse::<serde_json::Value>()?;
        let achievements_value =
            value
                .get("achievements")
                .ok_or_else(|| GamploError::MissingField {
                    field: "achievements".to_string(),
                    response: format!("{:?}", value),
                })?;
        let achievements: Vec<achievement::Achievement> =
            serde_json::from_value(achievements_value.clone()).map_err(|err| {
                GamploError::Deserialization {
                    type_name: "achievements".to_string(),
                    data: format!("{:?}", achievements_value),
                    source: err,
                }
            })?;

        Ok(achievements)
    }
    /// Gets all save slots for this client.
    pub async fn get_saves(&self) -> Result<Saves, GamploError> {
        let value = self
            .client
            .get(evaluate_url_path("/api/sdk/saves"))
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?;
        let saves: Saves =
            serde_json::from_str(&value).map_err(|err| GamploError::Deserialization {
                type_name: "Saves".to_string(),
                data: value.clone(),
                source: err,
            })?;
        Ok(saves)
    }
    /// Gets a specific save slot for this client.
    pub async fn get_save(&self, slot: u32) -> Result<Option<SaveData>, GamploError> {
        let response = self
            .client
            .get(evaluate_url_path("/api/sdk/saves"))
            .query(&[("slot", slot.to_string())])
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let text = response.text().await?;
        let save: SaveData =
            serde_json::from_str(&text).map_err(|err| GamploError::Deserialization {
                type_name: "SaveData".to_string(),
                data: text.clone(),
                source: err,
            })?;
        Ok(Some(save))
    }
    /// Unlocks an achievement for this client.
    pub async fn unlock_achievement(
        &self,
        achievement: &str,
    ) -> Result<AchievementUnlockResponse, GamploError> {
        let response = self
            .client
            .post("https://gamplo.com/api/sdk/achievements/unlock")
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone())
            .body(
                json!({
                    "key": achievement
                })
                .to_string(),
            )
            .send()
            .await?
            .text()
            .await?;

        let parsed: serde_json::Value = serde_json::from_str(&response)?;
        if parsed.get("success").and_then(|v| v.as_bool()) != Some(true) {
            return Err(GamploError::ApiError(format!(
                "Failed to unlock achievement: {}, response: {:?}",
                achievement, parsed
            )));
        }
        let response: AchievementUnlockResponse = serde_json::from_value(parsed)?;
        Ok(response)
    }
    /// Unlocks an achievement for this client with an API secret. For use on the server only as the API secret should never be exposed to clients.
    #[cfg(feature = "server")]
    pub async fn unlock_achievement_with_secret(
        &self,
        achievement: &str,
        api_secret: &str,
    ) -> Result<AchievementUnlockResponse, GamploError> {
        let req = self
            .client
            .post("https://gamplo.com/api/sdk/achievements/unlock")
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone())
            .header("x-api-secret", api_secret.to_string());
        let body = json!({ "key": achievement }).to_string();
        let text = req.body(body).send().await?.text().await?;
        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        if parsed.get("success").and_then(|v| v.as_bool()) != Some(true) {
            return Err(GamploError::ApiError(format!(
                "Failed to unlock achievement: {}, response: {:?}",
                achievement, parsed
            )));
        }
        Ok(serde_json::from_value(parsed)?)
    }
    /// Saves data to a specific slot for this client. If `slot` is `None`, it will save to the first available slot.
    pub async fn save(
        &self,
        slot: Option<u32>,
        data: serde_json::Value,
    ) -> Result<SaveWriteResponse, GamploError> {
        let mut body = json!({ "data": data });
        if let Some(s) = slot {
            body["slot"] = serde_json::json!(s);
        }
        let text = self
            .client
            .post(evaluate_url_path("/api/sdk/saves"))
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone())
            .body(body.to_string())
            .send()
            .await?
            .text()
            .await?;
        let resp: save::SaveWriteResponse =
            serde_json::from_str(&text).map_err(|e| GamploError::Deserialization {
                type_name: "SaveWriteResponse".to_string(),
                data: text.clone(),
                source: e,
            })?;
        Ok(resp)
    }
    /// Deletes a save slot for this client.
    pub async fn delete_save(&self, slot: u32) -> Result<save::SaveDeleteResponse, GamploError> {
        let text = self
            .client
            .delete(evaluate_url_path("/api/sdk/saves"))
            .query(&[("slot", slot.to_string())])
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?;
        let resp: save::SaveDeleteResponse =
            serde_json::from_str(&text).map_err(|e| GamploError::Deserialization {
                type_name: "SaveDeleteResponse".to_string(),
                data: text.clone(),
                source: e,
            })?;
        Ok(resp)
    }
    /// Moderates text for this client. Returns whether the text is allowed or blocked, and if blocked, the reason why.
    pub async fn moderate(&self, text: &str) -> Result<ModerationResult, GamploError> {
        let body = json!({ "text": text }).to_string();
        let text = self
            .client
            .post(evaluate_url_path("/api/sdk/moderate"))
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone())
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        let resp = {
            let parsed: serde_json::Value = serde_json::from_str(&text)?;
            if parsed.get("blocked").and_then(|v| v.as_bool()) == Some(true) {
                ModerationResult::Blocked {
                    reason: parsed
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                }
            } else {
                ModerationResult::Allowed
            }
        };
        Ok(resp)
    }
    /// Returns the session ID for this client.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

/// Represents the result of a moderation check.
///
/// If the text is blocked, it includes an optional reason for why it was blocked. If the text is allowed, there is no reason.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum ModerationResult {
    Blocked { reason: Option<String> },
    Allowed,
}
impl ModerationResult {
    pub fn is_blocked(&self) -> bool {
        match self {
            ModerationResult::Blocked { .. } => true,
            ModerationResult::Allowed => false,
        }
    }

    pub fn reason(&self) -> Option<&String> {
        match self {
            ModerationResult::Blocked { reason } => reason.as_ref(),
            ModerationResult::Allowed => None,
        }
    }
}

/// Attempts to get the Gamplo authentication token.
///
/// This function always returns an error as token auto-detection is disabled.
/// Use [`Gamplo::from_token`] to provide a token explicitly.
pub fn get_token() -> Result<String, GamploError> {
    let window = web_sys::window().ok_or_else(|| GamploError::TokenNotFound(String::from("Failed to get window object")))?;
    let url = Reflect::get(&window, &JsValue::from_str("GAMPLO_TOKEN")).map_err(|_| GamploError::TokenNotFound(String::from("Failed to access GAMPLO_TOKEN from window")))?;
    if url.is_undefined() {
        return Err(GamploError::TokenNotFound(String::from("GAMPLO_TOKEN is not defined on the window object")));
    }
    let token = url.as_string().unwrap();
    Ok(token)
}

#[cfg(test)]
mod tests {
    use crate::save::SaveData;

    use super::*;

    #[test]
    fn save_serde() {
        let save = SaveData {
            slot: 1,
            data: serde_json::json!({"foo": "bar"}),
            size_bytes: 123,
            updated_at: chrono::Utc::now(),
        };
        let serialized = serde_json::to_string(&save).unwrap();
        println!("Serialized save: {}", serialized);
        let deserialized: SaveData = serde_json::from_str(&serialized).unwrap();
        println!("Deserialized save: {:?}", deserialized);
        assert_eq!(save, deserialized);
    }

    #[test]
    fn achievement_serde() {
        let achievement = achievement::Achievement {
            id: 1,
            key: "first_blood".to_string(),
            title: "First Blood".to_string(),
            description: "Unlock your first achievement".to_string(),
            icon_url: "https://example.com/icon.png".to_string(),
            points: 10,
            hidden: false,
            unlocked: true,
            unlocked_at: chrono::Utc::now(),
        };
        let serialized = serde_json::to_string(&achievement).unwrap();
        println!("Serialized achievement: {}", serialized);
        let deserialized: achievement::Achievement = serde_json::from_str(&serialized).unwrap();
        println!("Deserialized achievement: {:?}", deserialized);
        assert_eq!(achievement, deserialized);
    }

    #[test]
    fn auth_player_nullable() {
        #[derive(serde::Deserialize)]
        struct AuthResponse {
            #[serde(rename = "sessionId")]
            session_id: String,
            player: Option<Player>,
        }

        let json_null = r#"{"sessionId":"abc","player":null}"#;
        let parsed: AuthResponse = serde_json::from_str(json_null).unwrap();
        assert_eq!(parsed.session_id, "abc");
        assert!(parsed.player.is_none());
    }

    #[test]
    fn auth_player_null_img() {
        #[derive(serde::Deserialize)]
        struct AuthResponse {
            #[serde(rename = "sessionId")]
            session_id: String,
            player: Option<Player>,
        }

        let json_null = "{\"sessionId\":\"nNr57QGzcg2737r9yGIDzIj1P1lM3UxV\",\"player\":{\"id\":\"hUPJNgTMQehHytsdX4ahST9B0x5HQJ5R\",\"username\":\"jay\",\"displayName\":\"jay\",\"image\":null}}";
        let parsed: AuthResponse = serde_json::from_str(json_null).unwrap();
        assert_eq!(parsed.session_id, "nNr57QGzcg2737r9yGIDzIj1P1lM3UxV");
        let player = parsed.player.unwrap();
        assert_eq!(player.id, "hUPJNgTMQehHytsdX4ahST9B0x5HQJ5R");
        assert_eq!(player.username, "jay");
        assert_eq!(player.display_name, "jay");
        assert!(player.avatar_url.is_none());
    }
}
