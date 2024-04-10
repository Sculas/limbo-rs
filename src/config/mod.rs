use figment::{
    providers::{Format, Toml},
    Figment,
};
use tracing::*;

static mut CONFIG: Option<Config> = None;

pub fn get() -> &'static Config {
    unsafe { CONFIG.as_ref().expect("Config uninitialized") }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// IP address to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// Maximum number of players allowed on the server
    pub max_players: i32,
    /// Message of the day to display in the server list
    pub motd: String,
    /// Version to display in the server list
    pub version: String,
    /// Server brand to display in F3 menu
    pub brand: String,
    /// Whether the limbo is behind Velocity and should use modern velocity forwarding
    pub uses_velocity_modern: bool,
    /// If `uses_velocity_modern` is true, the secret to use for verifying Velocity forwarding
    pub velocity_forwarding_secret: String,
    /// Whether the player's IP address should be withheld in the console
    pub hide_player_ips: bool,
}

impl Config {
    #[tracing::instrument]
    pub fn load(path: &str) -> anyhow::Result<&Config> {
        debug!("Loading configuration file");
        if std::fs::metadata(path).is_err() {
            info!("Configuration file not found, creating default");
            std::fs::write(path, include_str!("default.toml"))?;
        }
        let config = Figment::from(Toml::file_exact(path)).extract()?;
        unsafe { CONFIG = Some(config) };
        Ok(get())
    }
}
