use poise::serenity_prelude as serenity;

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn mute(
    ctx: super::Context<'_>,
    #[description = "Member to mute"] mut member: serenity::Member,
    #[description = "Duration in minutes"] duration: u64,
    #[description = "Reason"] reason: Option<String>,
) -> Result<(), anyhow::Error> {
    let reason = reason.unwrap_or_else(|| "No reason provided".to_string());
    let until = chrono::Utc::now() + chrono::Duration::minutes(duration as i64);
    let timeout = serenity::Timestamp::from_unix_timestamp(until.timestamp()).unwrap();

    member
        .disable_communication_until_datetime(&ctx.http(), timeout)
        .await?;

    ctx.say(format!(
        "🔇 Muted **{}** for {} minute(s). Reason: {}",
        member.user.name,
        duration,
        reason
    ))
    .await?;

    Ok(())
}

#[poise::command(slash_command, required_permissions = "MODERATE_MEMBERS")]
pub async fn unmute(
    ctx: super::Context<'_>,
    #[description = "Member to unmute"] mut member: serenity::Member,
) -> Result<(), anyhow::Error> {
    let now = serenity::Timestamp::from_unix_timestamp(chrono::Utc::now().timestamp()).unwrap();
    member.disable_communication_until_datetime(&ctx.http(), now).await?;

    ctx.say(format!("🔊 Unmuted **{}**", member.user.name))
        .await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn memberinfo(
    ctx: super::Context<'_>,
    #[description = "Member to look up"] member: Option<serenity::Member>,
) -> Result<(), anyhow::Error> {
    let member = if let Some(m) = member {
        m
    } else {
        ctx.author_member().await.unwrap().into_owned()
    };
    let user = &member.user;

    let guild_roles = ctx.guild_id().unwrap().roles(&ctx.http()).await?;
    let role_names: Vec<String> = member
        .roles
        .iter()
        .filter_map(|r| guild_roles.get(r).map(|role| role.name.clone()))
        .collect();

    let joined = member
        .joined_at
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let created = user.created_at().to_string();

    let display_name = member.display_name();

    let embed = serenity::CreateEmbed::default()
        .title(format!("Member Info — {}", user.name))
        .color(0x5865F2)
        .field("ID", &user.id.to_string(), true)
        .field("Display Name", &*display_name, true)
        .field("Joined Server", &joined, true)
        .field("Account Created", &created, true)
        .field("Roles", &if role_names.is_empty() { "None".to_string() } else { role_names.join(", ") }, false)
        .thumbnail(user.face());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn chatsummary(
    ctx: super::Context<'_>,
    #[description = "Number of messages (max 50)"] count: Option<u8>,
) -> Result<(), anyhow::Error> {
    let count = count.unwrap_or(20).min(50);
    let channel_id = ctx.channel_id();

    let msgs = channel_id
        .messages(&ctx.http(), serenity::GetMessages::new().limit(count))
        .await?;

    if msgs.is_empty() {
        ctx.say("No messages found.").await?;
        return Ok(());
    }

    let mut lines: Vec<String> = msgs
        .iter()
        .rev()
        .take(count as usize)
        .map(|m| {
            let content = if m.content.is_empty() {
                "[attachment or embed]"
            } else {
                &m.content
            };
            let clean = content.replace('\n', " ");
            let truncated = if clean.len() > 80 {
                format!("{}...", &clean[..80])
            } else {
                clean
            };
            format!("**{}**: {}", m.author.name, truncated)
        })
        .collect();

    lines.insert(0, format!("📋 **Chat Summary** — last {} messages\n", count));

    for chunk in lines.chunks(15) {
        ctx.say(chunk.join("\n")).await?;
    }

    Ok(())
}
