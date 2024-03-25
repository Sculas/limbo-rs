use std::sync::Arc;

use crate::player::Player;
use anyhow::Context;
use tokio::{net::TcpListener, sync::RwLock};
use tracing::*;

pub type AServer = Arc<Server>;

pub struct Server {
    listener: TcpListener,
    players: RwLock<Vec<Player>>,
}

impl Server {
    pub async fn bind(host: &str, port: u16) -> anyhow::Result<AServer> {
        // Bind to the specified host and port
        let listener = TcpListener::bind((host, port))
            .await
            .context(format!("Unable to bind to {host}:{port}"))?;
        // Return the server instance
        Ok(Arc::new(Self {
            listener,
            players: Default::default(),
        }))
    }

    pub async fn listen(self: &AServer) -> anyhow::Result<()> {
        debug!("Listening for incoming connections");
        loop {
            // Wait for incoming connections...
            let (stream, addr) = self
                .listener
                .accept()
                .await
                .context("Failed to accept connection")?;
            // When a connection is made, set the TCP_NODELAY option
            stream
                .set_nodelay(true)
                .context("Failed to initialize connection")?;
            // Then, spawn a new task to handle the connection
            let this = Arc::clone(&self);
            // handle_connection is implemented in network/connection.rs
            tokio::spawn(async move { this.handle_connection(stream, addr).await });
        }
    }

    pub async fn add_player(self: &AServer, player: Player) {
        let mut players_lock = self.players.write().await;
        players_lock.push(player);
    }

    pub async fn get_player_count(self: &AServer) -> usize {
        let players_lock = self.players.read().await;
        players_lock.len()
    }
}
