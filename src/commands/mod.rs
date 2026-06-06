mod identity;
mod moderation;
mod network;
mod player_stats;
mod system;
mod time_date;

use crate::Data;

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

pub fn commands() -> Vec<poise::Command<Data, anyhow::Error>> {
    vec![
        time_date::time(),
        time_date::timezones(),
        moderation::mute(),
        moderation::unmute(),
        moderation::memberinfo(),
        moderation::chatsummary(),
        player_stats::playtime(),
        player_stats::infractions(),
        identity::roblox(),
        identity::discord(),
        network::ping(),
        network::portscan(),
        network::ipintel(),
        system::sysinfo(),
    ]
}
