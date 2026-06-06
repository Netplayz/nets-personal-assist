use std::net::{IpAddr, ToSocketAddrs};
use std::time::{Duration, Instant};

use poise::serenity_prelude as serenity;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::services::ip_intel::IpIntelClient;
use crate::services::scanner;

#[poise::command(slash_command)]
pub async fn ping(
    ctx: super::Context<'_>,
    #[description = "Hostname or IP to ping"] host: String,
) -> Result<(), anyhow::Error> {
    ctx.defer().await?;

    let start = Instant::now();
    let resolved = resolve_host(&host);
    let resolve_time = start.elapsed();

    let resolved = match resolved {
        Ok(addr) => addr,
        Err(e) => {
            ctx.say(format!("❌ DNS resolution failed: {}", e)).await?;
            return Ok(());
        }
    };

    let start = Instant::now();
    let reachable = timeout(
        Duration::from_secs(5),
        TcpStream::connect((resolved, 80)),
    )
    .await
    .is_ok();
    let elapsed = start.elapsed();

    let status = if reachable {
        format!("✅ **Reachable** — {} ms", elapsed.as_millis())
    } else {
        "❌ **Unreachable** (connection timed out)".to_string()
    };

    let embed = serenity::CreateEmbed::default()
        .title(format!("🌐 Ping — {}", host))
        .field("IP Address", &resolved.to_string(), true)
        .field("DNS Resolution", format!("{} ms", resolve_time.as_millis()), true)
        .field("TCP Port 80", &status, false)
        .color(if reachable { 0x00FF00 } else { 0xFF0000 });

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn portscan(
    ctx: super::Context<'_>,
    #[description = "Hostname or IP"] host: String,
    #[description = "Scan mode (common / range)"] mode: Option<String>,
    #[description = "Start port (range mode only)"] start_port: Option<u16>,
    #[description = "End port (range mode only)"] end_port: Option<u16>,
) -> Result<(), anyhow::Error> {
    ctx.defer().await?;

    let ip = match resolve_host(&host) {
        Ok(ip) => ip,
        Err(e) => {
            ctx.say(format!("❌ DNS resolution failed: {}", e)).await?;
            return Ok(());
        }
    };

    let is_range = mode.as_deref() == Some("range");

    let open_ports = if is_range {
        let start = start_port.unwrap_or(1);
        let end = end_port.unwrap_or(1024);
        if end < start || end - start > 5000 {
            ctx.say("❌ Port range too large (max 5000).").await?;
            return Ok(());
        }
        scanner::scan_port_range(ip, start, end, 500, 200).await
    } else {
        scanner::scan_common_ports(ip, 500, 50).await
    };

    if open_ports.is_empty() {
        ctx.say(format!("🔍 **{}** — No open ports found.", ip))
            .await?;
        return Ok(());
    }

    let lines: Vec<String> = open_ports
        .iter()
        .map(|p| format!("`{:5}` — {}", p.port, p.service))
        .collect();

    let header = format!(
        "🔍 **Port Scan — {}** ({} open)\n",
        ip,
        open_ports.len()
    );

    for chunk in lines.chunks(20) {
        ctx.say(format!("{}\n{}", header, chunk.join("\n"))).await?;
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn ipintel(
    ctx: super::Context<'_>,
    #[description = "IP address"] ip: String,
) -> Result<(), anyhow::Error> {
    ctx.defer().await?;

    let addr: IpAddr = match ip.parse() {
        Ok(a) => a,
        Err(_) => {
            ctx.say("❌ Invalid IP address.").await?;
            return Ok(());
        }
    };

    let data = ctx.data();
    let client = IpIntelClient::new(
        data.config.ipinfo_token.clone(),
        data.config.abuseipdb_token.clone(),
    );

    let info = client.ipinfo_lookup(addr, Some(&data.cache)).await?;
    let mut embed = serenity::CreateEmbed::default()
        .title(format!("🌍 IP Intel — {}", info.ip))
        .field("City", info.city.as_deref().unwrap_or("N/A"), true)
        .field("Region", info.region.as_deref().unwrap_or("N/A"), true)
        .field("Country", info.country.as_deref().unwrap_or("N/A"), true)
        .field("Location", info.loc.as_deref().unwrap_or("N/A"), true)
        .field("ISP", info.org.as_deref().unwrap_or("N/A"), true)
        .field("Timezone", info.timezone.as_deref().unwrap_or("N/A"), true)
        .color(0x9B59B6);

    if data.config.abuseipdb_token.is_some() {
        match client.abuseipdb_lookup(&ip).await {
            Ok(report) => {
                embed = embed.field(
                    "Abuse Confidence",
                    format!("{}%", report.abuse_confidence_score),
                    true,
                );
                embed = embed.field("Total Reports", &report.total_reports.to_string(), true);
                embed = embed.field("ISP (AbuseIPDB)", &report.isp, true);
            }
            Err(_) => {}
        }
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn resolve_host(host: &str) -> anyhow::Result<IpAddr> {
    let addr = (host, 0)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve host"))?;
    Ok(addr.ip())
}
