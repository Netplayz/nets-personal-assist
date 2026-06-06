#![allow(dead_code)]

use sqlx::SqlitePool;

use super::models::{Infraction, Mute, Player};

pub async fn init_db(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            roblox_id INTEGER NOT NULL UNIQUE,
            discord_id INTEGER,
            username TEXT NOT NULL,
            playtime_seconds INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS infractions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER NOT NULL,
            infraction_type TEXT NOT NULL,
            reason TEXT NOT NULL,
            moderator_id INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (player_id) REFERENCES players(id)
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mutes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            discord_id INTEGER NOT NULL,
            channel_id INTEGER,
            reason TEXT NOT NULL,
            moderator_id INTEGER NOT NULL,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn upsert_player(pool: &SqlitePool, roblox_id: i64, username: &str) -> anyhow::Result<Player> {
    sqlx::query_as::<_, Player>(
        "INSERT INTO players (roblox_id, username)
         VALUES (?, ?)
         ON CONFLICT(roblox_id) DO UPDATE SET
            username = excluded.username,
            updated_at = datetime('now')
         RETURNING *",
    )
    .bind(roblox_id)
    .bind(username)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn get_player_by_roblox(pool: &SqlitePool, roblox_id: i64) -> anyhow::Result<Option<Player>> {
    sqlx::query_as::<_, Player>("SELECT * FROM players WHERE roblox_id = ?")
        .bind(roblox_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn get_player_by_discord(pool: &SqlitePool, discord_id: i64) -> anyhow::Result<Option<Player>> {
    sqlx::query_as::<_, Player>("SELECT * FROM players WHERE discord_id = ?")
        .bind(discord_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn add_playtime(pool: &SqlitePool, player_id: i64, seconds: i64) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE players SET playtime_seconds = playtime_seconds + ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(seconds)
    .bind(player_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_infraction(
    pool: &SqlitePool,
    player_id: i64,
    infraction_type: &str,
    reason: &str,
    moderator_id: i64,
) -> anyhow::Result<Infraction> {
    sqlx::query_as::<_, Infraction>(
        "INSERT INTO infractions (player_id, infraction_type, reason, moderator_id)
         VALUES (?, ?, ?, ?)
         RETURNING *",
    )
    .bind(player_id)
    .bind(infraction_type)
    .bind(reason)
    .bind(moderator_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn get_infractions(pool: &SqlitePool, player_id: i64) -> anyhow::Result<Vec<Infraction>> {
    sqlx::query_as::<_, Infraction>(
        "SELECT * FROM infractions WHERE player_id = ? ORDER BY created_at DESC",
    )
    .bind(player_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub async fn add_mute(
    pool: &SqlitePool,
    discord_id: i64,
    channel_id: Option<i64>,
    reason: &str,
    moderator_id: i64,
    expires_at: &str,
) -> anyhow::Result<Mute> {
    sqlx::query_as::<_, Mute>(
        "INSERT INTO mutes (discord_id, channel_id, reason, moderator_id, expires_at)
         VALUES (?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(discord_id)
    .bind(channel_id)
    .bind(reason)
    .bind(moderator_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn get_active_mutes(pool: &SqlitePool) -> anyhow::Result<Vec<Mute>> {
    sqlx::query_as::<_, Mute>(
        "SELECT * FROM mutes WHERE datetime(expires_at) > datetime('now') ORDER BY expires_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
