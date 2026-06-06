use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use super::cache::AppCache;

pub struct IpIntelClient {
    client: reqwest::Client,
    ipinfo_token: Option<String>,
    abuseipdb_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    pub ip: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub loc: Option<String>,
    pub org: Option<String>,
    pub postal: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbuseIPDBReport {
    pub ip_address: String,
    pub abuse_confidence_score: i64,
    pub country_code: String,
    pub usage_type: String,
    pub isp: String,
    pub domain: String,
    pub total_reports: i64,
    pub last_reported_at: String,
}

impl IpIntelClient {
    pub fn new(ipinfo_token: Option<String>, abuseipdb_token: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .gzip(true)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            ipinfo_token,
            abuseipdb_token,
        }
    }

    pub async fn ipinfo_lookup(
        &self,
        ip: IpAddr,
        cache: Option<&AppCache>,
    ) -> anyhow::Result<IpInfo> {
        if let Some(c) = cache {
            if let Some(cached) = c.ip_info.get(&ip).await {
                return serde_json::from_str(&cached).map_err(Into::into);
            }
        }

        let url = if let Some(ref token) = self.ipinfo_token {
            format!("https://ipinfo.io/{}?token={}", ip, token)
        } else {
            format!("https://ipinfo.io/{}/json", ip)
        };

        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("IP info lookup failed: {}", resp.status());
        }

        let info: IpInfo = resp.json().await?;

        if let Some(c) = cache {
            let serialized = serde_json::to_string(&info)?;
            c.ip_info.insert(ip, serialized).await;
        }

        Ok(info)
    }

    pub async fn abuseipdb_lookup(&self, ip: &str) -> anyhow::Result<AbuseIPDBReport> {
        let token = self
            .abuseipdb_token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("ABUSEIPDB_TOKEN not configured"))?;

        let resp = self
            .client
            .get("https://api.abuseipdb.com/api/v2/check")
            .query(&[("ipAddress", ip), ("maxAgeInDays", "90")])
            .header("Key", token)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("AbuseIPDB lookup failed: {}", resp.status());
        }

        #[derive(Deserialize)]
        struct Wrapper {
            data: AbuseIPDBReport,
        }

        let wrapper: Wrapper = resp.json().await?;
        Ok(wrapper.data)
    }
}
