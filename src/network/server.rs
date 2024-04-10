use std::sync::Arc;

use crate::player::Player;
use anyhow::Context;
use dashmap::DashMap;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::*;

pub type AServer = Arc<Server>;

pub struct Server {
    listener: TcpListener,
    players: DashMap<uuid::Uuid, Mutex<Player>>,
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

    pub fn add_player(self: &AServer, player: Player) {
        self.players.insert(player.uuid, Mutex::new(player));
    }

    pub fn get_player<'a>(
        self: &'a AServer,
        uuid: uuid::Uuid,
    ) -> super::Result<dashmap::mapref::one::Ref<'a, uuid::Uuid, Mutex<Player>>> {
        match self.players.get(&uuid) {
            Some(player_ref) => Ok(player_ref),
            None => crate::network_bail!("Player data not found for UUID {uuid}"),
        }
    }

    pub fn remove_player(self: &AServer, uuid: uuid::Uuid) {
        self.players.remove(&uuid);
    }

    pub fn get_player_count(self: &AServer) -> usize {
        self.players.len()
    }
}
