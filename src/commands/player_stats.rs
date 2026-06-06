use crate::db::queries;
use crate::utils::time;

#[poise::command(slash_command)]
pub async fn playtime(
    ctx: super::Context<'_>,
    #[description = "Roblox user ID"] roblox_id: i64,
) -> Result<(), anyhow::Error> {
    let data = ctx.data();
    let player = queries::get_player_by_roblox(&data.pool, roblox_id).await?;

    let player = match player {
        Some(p) => p,
        None => {
            ctx.say(format!(
                "Player with Roblox ID `{}` not found in database.",
                roblox_id
            ))
            .await?;
            return Ok(());
        }
    };

    let formatted = time::format_duration_seconds(player.playtime_seconds);
    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title(format!("Playtime — {}", player.username))
        .field("Roblox ID", &player.roblox_id.to_string(), true)
        .field("Playtime", &formatted, true)
        .field("First Seen", &player.created_at, true)
        .field("Last Updated", &player.updated_at, true)
        .color(0x00FF00);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn infractions(
    ctx: super::Context<'_>,
    #[description = "Roblox user ID"] roblox_id: i64,
) -> Result<(), anyhow::Error> {
    let data = ctx.data();
    let player = queries::get_player_by_roblox(&data.pool, roblox_id).await?;

    let player = match player {
        Some(p) => p,
        None => {
            ctx.say(format!(
                "Player with Roblox ID `{}` not found.",
                roblox_id
            ))
            .await?;
            return Ok(());
        }
    };

    let infractions = queries::get_infractions(&data.pool, player.id).await?;

    if infractions.is_empty() {
        ctx.say(format!("**{}** has no infractions. ✅", player.username))
            .await?;
        return Ok(());
    }

    let mut lines: Vec<String> = infractions
        .iter()
        .enumerate()
        .map(|(i, inf)| {
            format!(
                "{}. **{}** — {} (by <@{}>) — {}",
                i + 1,
                inf.infraction_type,
                inf.reason,
                inf.moderator_id,
                inf.created_at
            )
        })
        .collect();

    lines.insert(
        0,
        format!("📋 **Infractions for {}** ({} total)\n", player.username, infractions.len()),
    );

    for chunk in lines.chunks(15) {
        ctx.say(chunk.join("\n")).await?;
    }

    Ok(())
}
