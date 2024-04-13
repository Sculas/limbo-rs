use clap::Parser;
use network::server::Server;
use tracing::*;
use tracing_subscriber::EnvFilter;

mod config;
mod network;
mod player;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, propagate_version = true)]
struct Args {
    /// Path to the configuration file
    #[arg(long, short, default_value = "./config.toml")]
    config_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info,limbo=trace"))
        .try_init()
        .expect("Failed to initialize tracing");

    let config = config::Config::load(&args.config_path)?;
    if !config.uses_velocity_modern {
        // TODO: Fetch player skins from Mojang API
        warn!("This server is running in offline mode. Skins will not load (yet), and anyone can join with any username.");
    }
    let server = Server::bind(&config.host, config.port).await?;
    info!("Starting server on {}:{}", config.host, config.port);
    server.listen().await
}
