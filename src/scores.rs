use crate::config::Mode;
use anyhow::{Result, bail};
use reqwest::{StatusCode, blocking::Client};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Score {
    pub created_at: String,
    pub beatmap: Beatmap,
    pub beatmapset: Beatmapset,
}

#[derive(Deserialize)]
pub struct Beatmap {
    pub total_length: u32,
}

#[derive(Deserialize)]
pub struct Beatmapset {
    pub artist: String,
    pub artist_unicode: String,
    pub title: String,
    pub title_unicode: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

pub fn fetch_token(client_id: u32, client_secret: &str) -> Result<String> {
    let res = Client::new()
        .post("https://osu.ppy.sh/oauth/token")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "grant_type": "client_credentials",
            "scope": "public"
        }))
        .send()
        .map_err(|e| anyhow::anyhow!("Could not request token: {e}"))?;

    if res.status() != StatusCode::OK {
        bail!("Could not fetch bearer token, Received status code: {}", res.status());
    }

    let token: TokenResponse = res.json().map_err(|e| anyhow::anyhow!("Could not deserialize token response: {e}"))?;

    Ok(token.access_token)
}

pub fn get_recent_score(user_id: u64, mode: &Option<Mode>, token: &str) -> Result<Option<Score>> {
    let mut request = Client::new().get(format!("https://osu.ppy.sh/api/v2/users/{}/scores/recent", user_id)).bearer_auth(token);

    if let Some(mode) = mode {
        request = request.query(&[("mode", mode)]);
    }

    let response = match request.send() {
        Ok(response) => response,
        Err(error) => bail!("Could not send request to get user's recent score: {error}"),
    };

    let status_code = response.status();

    if status_code != StatusCode::OK {
        bail!("Could not get user's recent score. Received status code: {status_code}");
    }

    let Ok(mut scores) = response.json::<Vec<Score>>() else { return Ok(None) };

    if scores.is_empty() { Ok(None) } else { Ok(Some(scores.remove(0))) }
}
