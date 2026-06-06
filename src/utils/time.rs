use chrono::{DateTime, Utc};
use chrono_tz::Tz;

pub fn now_in_timezone(tz_str: &str) -> anyhow::Result<String> {
    let tz: Tz = tz_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid timezone: {}", tz_str))?;
    let now: DateTime<Utc> = Utc::now();
    let local = now.with_timezone(&tz);
    Ok(local.format("%Y-%m-%d %H:%M:%S %Z").to_string())
}

pub fn list_timezones(query: &str) -> Vec<String> {
    let q = query.to_lowercase();
    chrono_tz::TZ_VARIANTS
        .iter()
        .filter(|tz| tz.name().to_lowercase().contains(&q))
        .take(10)
        .map(|tz| tz.name().to_string())
        .collect()
}

pub fn format_duration_seconds(secs: i64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
}
