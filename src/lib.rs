pub mod achievement;
pub mod player;
pub mod save;

use anyhow::anyhow;
use serde_json::json;

use crate::{
    achievement::AchievementUnlockResponse,
    player::Player,
    save::{SaveData, Saves},
};

pub const URL: &str = "https://gamplo.com";
fn base_url() -> String {
    if let Ok(env) = std::env::var("GAMPLO_BASE_URL") {
        return env;
    }
    // On WASM, allow `?gamplo_base=...` override for local proxy/dev
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let location = window.location();
            if let Ok(search) = location.search() {
                let query = url::form_urlencoded::parse(search.as_bytes()).collect::<Vec<_>>();
                if let Some((_, v)) = query.into_iter().find(|(k, _)| k == "gamplo_base") {
                    return v.into_owned();
                }
            }
        }
    }
    URL.to_string()
}
pub fn url_path(path: &str) -> String {
    base_url() + path
}
#[derive(Debug, Clone)]
pub struct GamploClient {
    session_id: String,
    client: reqwest::Client,
}
impl GamploClient {
    pub async fn from_token(token: String) -> anyhow::Result<Self> {
        Ok(Self::from_token_with_player(token).await?.0)
    }

    pub async fn from_token_with_player(token: String) -> anyhow::Result<(Self, Option<Player>)> {
        let client = reqwest::Client::new();
        let text = client
            .post(url_path("/api/sdk/auth"))
            .header("Content-Type", "application/json")
            .body(json!({ "token": token }).to_string())
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send auth request: {:?}", e))?
            .text()
            .await
            .map_err(|e| anyhow!("Error receiving text: {:?}", e))?;

        #[derive(serde::Deserialize)]
        struct AuthResponse {
            #[serde(rename = "sessionId")]
            session_id: String,
            player: Option<Player>,
        }

        let parsed: AuthResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse auth response: {:?}, error: {:?}", text, e))?;

        let client_struct = GamploClient {
            session_id: parsed.session_id,
            client,
        };
        Ok((client_struct, parsed.player))
    }
    pub async fn new() -> anyhow::Result<Self> {
        let token = get_token()?;
        Self::from_token(token).await
    }

    pub async fn new_with_player() -> anyhow::Result<(Self, Option<Player>)> {
        let token = get_token()?;
        Self::from_token_with_player(token).await
    }
    pub async fn get_player(&self) -> anyhow::Result<Option<Player>> {
        let value = self
            .client
            .get(url_path("/api/sdk/player"))
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?
            .parse::<serde_json::Value>()?;

        let player_value = value
            .get("player")
            .ok_or_else(|| anyhow!("Failed to get player from response: {:?}", value))?;

        if player_value.is_null() {
            return Ok(None);
        }

        let player: Player = serde_json::from_value(player_value.clone()).map_err(|err| {
            anyhow!(
                "Failed to deserialize player: {:?}, error: {:?}",
                player_value,
                err
            )
        })?;

        Ok(Some(player))
    }
    pub async fn get_achievements(&self) -> anyhow::Result<Vec<achievement::Achievement>> {
        let value = self
            .client
            .get(url_path("/api/sdk/achievements"))
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?
            .parse::<serde_json::Value>()?;
        let achievements_value = value
            .get("achievements")
            .ok_or_else(|| anyhow!("Failed to get achievements from response: {:?}", value))?;
        let achievements: Vec<achievement::Achievement> =
            serde_json::from_value(achievements_value.clone()).map_err(|err| {
                anyhow!(
                    "Failed to deserialize achievements: {:?}, error: {:?}",
                    achievements_value,
                    err
                )
            })?;

        Ok(achievements)
    }
    pub async fn get_saves(&self) -> anyhow::Result<Saves> {
        let value = self
            .client
            .get(url_path("/api/sdk/saves"))
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?;
        let saves: Saves = serde_json::from_str(&value).map_err(|err| {
            anyhow!(
                "Failed to deserialize saves response: {:?}, error: {:?}",
                value,
                err
            )
        })?;
        Ok(saves)
    }
    pub async fn get_save(&self, slot: u32) -> anyhow::Result<Option<SaveData>> {
        let response = self
            .client
            .get(url_path("/api/sdk/saves"))
            .query(&[("slot", slot.to_string())])
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let text = response.text().await?;
        let save: SaveData = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(err) => {
                return Err(anyhow!(
                    "Failed to deserialize save: {:?}, error: {:?}",
                    text,
                    err
                ));
            }
        };
        Ok(Some(save))
    }
    pub async fn unlock(&self, achievement: &str) -> anyhow::Result<AchievementUnlockResponse> {
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
            return Err(anyhow!(
                "Failed to unlock achievement: {}, response: {:?}",
                achievement,
                parsed
            ));
        }
        let response: AchievementUnlockResponse = serde_json::from_value(parsed)?;
        Ok(response)
    }

    pub async fn unlock_with_secret(
        &self,
        achievement: &str,
        api_secret: Option<&str>,
    ) -> anyhow::Result<AchievementUnlockResponse> {
        let mut req = self
            .client
            .post("https://gamplo.com/api/sdk/achievements/unlock")
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone());
        if let Some(secret) = api_secret {
            req = req.header("x-api-secret", secret.to_string());
        }
        let body = json!({ "key": achievement }).to_string();
        let text = req.body(body).send().await?.text().await?;
        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        if parsed.get("success").and_then(|v| v.as_bool()) != Some(true) {
            return Err(anyhow!(
                "Failed to unlock achievement: {}, response: {:?}",
                achievement,
                parsed
            ));
        }
        Ok(serde_json::from_value(parsed)?)
    }

    pub async fn save(
        &self,
        slot: Option<u32>,
        data: serde_json::Value,
    ) -> anyhow::Result<save::SaveWriteResponse> {
        let mut body = json!({ "data": data });
        if let Some(s) = slot {
            body["slot"] = serde_json::json!(s);
        }
        let text = self
            .client
            .post(url_path("/api/sdk/saves"))
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone())
            .body(body.to_string())
            .send()
            .await?
            .text()
            .await?;
        let resp: save::SaveWriteResponse = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse save response: {:?}, error: {:?}", text, e))?;
        Ok(resp)
    }

    pub async fn delete_save(&self, slot: u32) -> anyhow::Result<save::SaveDeleteResponse> {
        let text = self
            .client
            .delete(url_path("/api/sdk/saves"))
            .query(&[("slot", slot.to_string())])
            .header("x-sdk-session", self.session_id.clone())
            .send()
            .await?
            .text()
            .await?;
        let resp: save::SaveDeleteResponse = serde_json::from_str(&text).map_err(|e| {
            anyhow!(
                "Failed to parse delete response: {:?}, error: {:?}",
                text,
                e
            )
        })?;
        Ok(resp)
    }

    pub async fn moderate(&self, text: &str) -> anyhow::Result<ModerationResult> {
        let body = json!({ "text": text }).to_string();
        let text = self
            .client
            .post(url_path("/api/sdk/moderate"))
            .header("Content-Type", "application/json")
            .header("x-sdk-session", self.session_id.clone())
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        let resp: ModerationResult = serde_json::from_str(&text).map_err(|e| {
            anyhow!(
                "Failed to parse moderation response: {:?}, error: {:?}",
                text,
                e
            )
        })?;
        Ok(resp)
    }

    // Expose the current session ID for diagnostics/status UIs.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationResult {
    pub blocked: bool,
    pub reason: Option<String>,
}

pub fn get_token() -> anyhow::Result<String> {
    if let Ok(token) = std::env::var("GAMPLO_TOKEN") {
        return Ok(token);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(anyhow!(
            "GAMPLO_TOKEN not set and not running in a browser (WASM)"
        ))
    }

    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let search = location
            .search()
            .map_err(|jsval| anyhow!("Failed to get search from location: {:?}", jsval))?;
        let query = url::form_urlencoded::parse(search.as_bytes()).collect::<Vec<_>>();
        let token = query.into_iter().find(|(key, _)| key == "gamplo_token").map(|(_, value)| value.into_owned()).ok_or_else(|| {
                anyhow!("GAMPLO_TOKEN environment variable not set and no token query parameter found in URL")
        })?;
        Ok(token)
    }
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
