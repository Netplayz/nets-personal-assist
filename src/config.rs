use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub database_url: String,
    pub roblox_cookie: Option<String>,
    pub ipinfo_token: Option<String>,
    pub abuseipdb_token: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            discord_token: env::var("DISCORD_TOKEN")
                .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN must be set"))?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data.db?mode=rwc".into()),
            roblox_cookie: env::var("ROBLOX_COOKIE").ok(),
            ipinfo_token: env::var("IPINFO_TOKEN").ok(),
            abuseipdb_token: env::var("ABUSEIPDB_TOKEN").ok(),
        })
    }
}
