#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Player {
    pub id: i64,
    pub roblox_id: i64,
    pub discord_id: Option<i64>,
    pub username: String,
    pub playtime_seconds: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Infraction {
    pub id: i64,
    pub player_id: i64,
    pub infraction_type: String,
    pub reason: String,
    pub moderator_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Mute {
    pub id: i64,
    pub discord_id: i64,
    pub channel_id: Option<i64>,
    pub reason: String,
    pub moderator_id: i64,
    pub expires_at: String,
    pub created_at: String,
}
