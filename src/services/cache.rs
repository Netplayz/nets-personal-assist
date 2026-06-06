use moka::future::Cache;
use std::{net::IpAddr, time::Duration};

use super::roblox_api::RobloxUser;

#[derive(Debug, Clone)]
pub struct AppCache {
    pub roblox_user: Cache<i64, RobloxUser>,
    pub roblox_avatar: Cache<i64, String>,
    pub ip_info: Cache<IpAddr, String>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            roblox_user: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(300))
                .time_to_idle(Duration::from_secs(60))
                .build(),
            roblox_avatar: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(600))
                .build(),
            ip_info: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(1800))
                .build(),
        }
    }
}
