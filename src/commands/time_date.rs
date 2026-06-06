use crate::utils::time;

#[poise::command(slash_command, prefix_command)]
pub async fn time(
    ctx: super::Context<'_>,
    #[description = "Timezone (e.g. America/New_York, Europe/London)"] timezone: Option<String>,
) -> Result<(), anyhow::Error> {
    let tz = timezone.unwrap_or_else(|| "UTC".to_string());
    match time::now_in_timezone(&tz) {
        Ok(result) => {
            ctx.say(format!("🕐 **Current time ({})**: {}", tz, result)).await?;
        }
        Err(_e) => {
            let suggestions = time::list_timezones(&tz);
            let msg = if suggestions.is_empty() {
                format!("Invalid timezone `{}`. Use `/timezones` to search.", tz)
            } else {
                format!(
                    "Invalid timezone `{}`. Did you mean: {}?",
                    tz,
                    suggestions.join(", ")
                )
            };
            ctx.say(msg).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn timezones(
    ctx: super::Context<'_>,
    #[description = "Search term"] query: Option<String>,
) -> Result<(), anyhow::Error> {
    let q = query.as_deref().unwrap_or("");
    let results = time::list_timezones(q);
    if results.is_empty() {
        ctx.say("No timezones found matching that query.").await?;
    } else {
        ctx.say(format!("**Matching timezones:**\n{}", results.join("\n")))
            .await?;
    }
    Ok(())
}
