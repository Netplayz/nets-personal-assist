use crate::services::sysinfo;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command)]
pub async fn sysinfo(ctx: super::Context<'_>) -> Result<(), anyhow::Error> {
    ctx.defer().await?;

    let snap = sysinfo::gather_snapshot();

    let uptime_days = snap.uptime / 86400;
    let uptime_hrs = (snap.uptime % 86400) / 3600;
    let uptime_min = (snap.uptime % 3600) / 60;

    let mem_total_gb = snap.total_memory as f64 / 1_073_741_824.0;
    let mem_used_gb = snap.used_memory as f64 / 1_073_741_824.0;
    let mem_pct = if snap.total_memory > 0 {
        (snap.used_memory as f64 / snap.total_memory as f64 * 100.0) as u64
    } else {
        0
    };

    let swap_total_gb = snap.total_swap as f64 / 1_073_741_824.0;
    let swap_used_gb = snap.used_swap as f64 / 1_073_741_824.0;

    let disk_lines: Vec<String> = snap
        .disk_info
        .iter()
        .map(|d| {
            let total_gb = d.total as f64 / 1_073_741_824.0;
            let used_gb = d.used as f64 / 1_073_741_824.0;
            let pct = if d.total > 0 {
                (d.used as f64 / d.total as f64 * 100.0) as u64
            } else {
                0
            };
            format!(
                "`{}` — {:.1} GB / {:.1} GB ({}%)",
                d.mount, used_gb, total_gb, pct
            )
        })
        .collect();

    let net_lines: Vec<String> = snap
        .network_info
        .iter()
        .map(|n| {
            let rx_mb = n.received as f64 / 1_048_576.0;
            let tx_mb = n.transmitted as f64 / 1_048_576.0;
            format!("`{}` — ⬇ {:.1} MB / ⬆ {:.1} MB", n.name, rx_mb, tx_mb)
        })
        .collect();

    let embed = serenity::CreateEmbed::default()
        .title("🖥️ System Information")
        .field("Hostname", snap.hostname, true)
        .field("OS", snap.os, true)
        .field("Kernel", snap.kernel, true)
        .field(
            "Uptime",
            format!("{}d {}h {}m", uptime_days, uptime_hrs, uptime_min),
            true,
        )
        .field(
            "CPU",
            format!(
                "{} ({} cores) — {:.1}%",
                snap.cpu_brand, snap.cpu_count, snap.cpu_usage
            ),
            false,
        )
        .field(
            "Memory",
            format!(
                "{:.1} GB / {:.1} GB ({}%)",
                mem_used_gb, mem_total_gb, mem_pct
            ),
            true,
        )
        .field(
            "Swap",
            format!("{:.1} GB / {:.1} GB", swap_used_gb, swap_total_gb),
            true,
        )
        .field("Disks", disk_lines.join("\n"), false)
        .field("Network", net_lines.join("\n"), false)
        .color(0x00FF00);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
