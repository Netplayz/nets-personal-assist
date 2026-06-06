#![allow(dead_code)]

use std::sync::OnceLock;

use serde::Deserialize;
use tokio::sync::Semaphore;

use super::cache::AppCache;

static ROBLOX_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn roblox_client() -> &'static reqwest::Client {
    ROBLOX_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("OWTF-Bot/1.0")
            .gzip(true)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to build reqwest client")
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct RobloxUser {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub created: String,
}

#[derive(Debug, Deserialize)]
struct RobloxUsersResponse {
    data: Vec<RobloxUser>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RobloxGame {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub playing: i64,
    pub visits: i64,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Deserialize)]
struct RobloxGameResponse {
    data: Vec<RobloxGame>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailResponse {
    data: Vec<ThumbnailItem>,
}

#[derive(Debug, Deserialize)]
struct ThumbnailItem {
    #[allow(dead_code)]
    image_url: String,
}

pub async fn lookup_user_by_id(
    user_id: i64,
    cache: Option<&AppCache>,
    semaphore: &Semaphore,
) -> anyhow::Result<RobloxUser> {
    if let Some(c) = cache {
        if let Some(cached) = c.roblox_user.get(&user_id).await {
            return Ok(cached);
        }
    }

    let _permit = semaphore.acquire().await?;
    let url = format!("https://users.roblox.com/v1/users/{}", user_id);
    let resp = roblox_client().get(&url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Roblox API returned {}", resp.status());
    }

    let user: RobloxUser = resp.json().await?;

    if let Some(c) = cache {
        c.roblox_user.insert(user_id, user.clone()).await;
    }

    Ok(user)
}

pub async fn lookup_user_by_username(
    username: &str,
    semaphore: &Semaphore,
) -> anyhow::Result<RobloxUser> {
    let _permit = semaphore.acquire().await?;
    let resp = roblox_client()
        .post("https://users.roblox.com/v1/usernames/users")
        .json(&serde_json::json!({
            "usernames": [username],
            "excludeBannedUsers": false
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Roblox API returned {}", resp.status());
    }

    let data: RobloxUsersResponse = resp.json().await?;
    data.data
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("User not found"))
}

pub async fn lookup_user_games(
    user_id: i64,
    semaphore: &Semaphore,
) -> anyhow::Result<Vec<RobloxGame>> {
    let _permit = semaphore.acquire().await?;
    let url = format!(
        "https://games.roblox.com/v2/users/{}/games?limit=10&sortOrder=Asc",
        user_id
    );
    let resp = roblox_client().get(&url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Roblox API returned {}", resp.status());
    }

    let data: RobloxGameResponse = resp.json().await?;
    Ok(data.data)
}

pub async fn get_user_avatar(
    user_id: i64,
    cache: Option<&AppCache>,
    semaphore: &Semaphore,
) -> anyhow::Result<String> {
    if let Some(c) = cache {
        if let Some(url) = c.roblox_avatar.get(&user_id).await {
            return Ok(url);
        }
    }

    let _permit = semaphore.acquire().await?;
    let url = format!(
        "https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds={}&size=420x420&format=Png",
        user_id
    );
    let resp = roblox_client().get(&url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!("Roblox thumbnails API returned {}", resp.status());
    }

    let data: ThumbnailResponse = resp.json().await?;
    let image_url = data
        .data
        .into_iter()
        .next()
        .map(|t| t.image_url)
        .ok_or_else(|| anyhow::anyhow!("No avatar found"))?;

    if let Some(c) = cache {
        c.roblox_avatar.insert(user_id, image_url.clone()).await;
    }

    Ok(image_url)
}
