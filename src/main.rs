mod commands;
mod config;
mod db;
mod services;
mod utils;

use db::queries::init_db;
use services::cache::AppCache;
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::Semaphore;
use tracing_subscriber::EnvFilter;

pub struct Data {
    pub pool: sqlx::SqlitePool,
    pub config: config::Config,
    pub cache: AppCache,
    pub roblox_semaphore: Semaphore,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("pool", &self.pool)
            .field("cache", &self.cache)
            .field("roblox_semaphore", &self.roblox_semaphore)
            .finish_non_exhaustive()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn")),
        )
        .init();

    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    tracing::info!("Connecting to database...");
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    // Enable WAL mode for concurrent read performance
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA cache_size=-8000")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    init_db(&pool).await?;
    tracing::info!("Database initialized.");

    let data = Data {
        pool,
        config: config.clone(),
        cache: AppCache::new(),
        roblox_semaphore: Semaphore::new(10),
    };

    let intents = poise::serenity_prelude::GatewayIntents::non_privileged()
        | poise::serenity_prelude::GatewayIntents::MESSAGE_CONTENT
        | poise::serenity_prelude::GatewayIntents::GUILD_MEMBERS
        | poise::serenity_prelude::GatewayIntents::GUILD_MODERATION;

    let discord_token = config.discord_token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            on_error: |error| {
                Box::pin(async move {
                    tracing::error!("Command error: {:?}", error);
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                tracing::info!("Bot is ready!");
                Ok(data)
            })
        })
        .build();

    let mut client = poise::serenity_prelude::ClientBuilder::new(
        &discord_token,
        intents,
    )
    .framework(framework)
    .await?;

    if let Err(why) = client.start().await {
        tracing::error!("Client error: {:?}", why);
    }

    Ok(())
}
