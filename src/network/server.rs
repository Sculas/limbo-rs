use std::sync::{atomic::AtomicU32, Arc};

use crate::{config, player::Player};
use anyhow::Context;
use dashmap::DashMap;
use tokio::{net::TcpListener, sync::Mutex};
use tracing::*;

pub mod constants {
    pub const VIEW_DISTANCE: u32 = 8;
    pub const SIMULATION_DISTANCE: u32 = 8;
}

pub type AServer = Arc<Server>;
pub type PlayerRef = Arc<Mutex<Player>>;

pub struct Server {
    listener: TcpListener,
    players: DashMap<uuid::Uuid, Arc<Mutex<Player>>>,
    entity_id_counter: AtomicU32,
}

impl Server {
    pub async fn bind(host: &str, port: u16) -> anyhow::Result<AServer> {
        // Read and initialize registry data
        crate::utils::registry_data::init()?;
        // Bind to the specified host and port
        let listener = TcpListener::bind((host, port))
            .await
            .context(format!("Unable to bind to {host}:{port}"))?;
        // Return the server instance
        Ok(Arc::new(Self {
            listener,
            players: Default::default(),
            entity_id_counter: Default::default(),
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

    pub fn add_player(self: &AServer, player: Player) -> super::Result<PlayerRef> {
        let uuid = player.uuid();
        self.players.insert(uuid, Arc::new(Mutex::new(player)));
        self.get_player(&uuid)
    }

    pub fn get_player(self: &AServer, uuid: &uuid::Uuid) -> super::Result<PlayerRef> {
        match self.players.get(uuid) {
            Some(player_ref) => Ok(Arc::clone(&player_ref)),
            None => crate::network_bail!("Player data not found for UUID {uuid}"),
        }
    }

    // TODO: Remove player on disconnect
    pub fn remove_player(self: &AServer, uuid: uuid::Uuid) {
        self.players.remove(&uuid);
    }

    /// Current player count, returned as an i32 (for protocol compatibility)
    pub fn get_player_count(self: &AServer) -> i32 {
        self.players.len() as i32
    }

    pub fn is_full(self: &AServer) -> bool {
        self.get_player_count() >= config::get().max_players
    }

    pub fn next_entity_id(self: &AServer) -> u32 {
        self.entity_id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}
