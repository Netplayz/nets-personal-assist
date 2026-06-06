use crate::services::roblox_api;

#[poise::command(slash_command)]
pub async fn roblox(
    ctx: super::Context<'_>,
    #[description = "Roblox user ID or username"] id_or_username: String,
) -> Result<(), anyhow::Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let sem = &data.roblox_semaphore;

    let user = if let Ok(id) = id_or_username.parse::<i64>() {
        roblox_api::lookup_user_by_id(id, Some(&data.cache), sem).await?
    } else {
        roblox_api::lookup_user_by_username(&id_or_username, sem).await?
    };

    let avatar = roblox_api::get_user_avatar(user.id, Some(&data.cache), sem)
        .await
        .unwrap_or_default();

    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title(format!("{} (@{})", user.display_name, user.name))
        .url(format!("https://www.roblox.com/users/{}/profile", user.id))
        .field("User ID", &user.id.to_string(), true)
        .field("Display Name", &user.display_name, true)
        .field("Description", &truncate(&user.description, 200), false)
        .field("Created", &user.created, true)
        .color(0xED4245)
        .thumbnail(&avatar);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn discord(
    ctx: super::Context<'_>,
    #[description = "Discord user"] user: poise::serenity_prelude::User,
) -> Result<(), anyhow::Error> {
    let created = user.created_at().to_string();

    let display_name = user.display_name();
    let is_bot = if user.bot { "Yes" } else { "No" };

    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title(format!("Discord User — {}", user.name))
        .field("ID", &user.id.to_string(), true)
        .field("Display Name", &*display_name, true)
        .field("Account Created", &created, true)
        .field("Bot", &*is_bot, true)
        .color(0x5865F2)
        .thumbnail(user.face());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
