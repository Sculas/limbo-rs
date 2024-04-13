use azalea_core::position::ChunkPos;
use azalea_protocol::packets::game::{
    clientbound_game_event_packet::EventType, ServerboundGamePacket,
};
use azalea_world::{Chunk, Section};
use tracing::*;

use crate::{
    bail_packet_error, config,
    network::{
        self,
        ext::ConnectionExt,
        server::{AServer, PlayerRef},
    },
};

mod utils;

/// Attempts to handle the game phase.
#[tracing::instrument(name = "game", skip_all)]
pub async fn try_handle(
    mut conn: network::GameConnection,
    server: &AServer,
    player: &PlayerRef,
) -> network::Result<()> {
    debug!("Handling game phase");

    // Initialize the player entity
    player.lock().await.init(server);
    // Signal game start to the client
    utils::signal_game_start(&mut conn, &player).await?;
    // Signal player update to the client
    utils::signal_player_update(&mut conn, &player).await?;
    // Signal spawn position to the client
    utils::signal_spawn_position(&mut conn).await?;
    // Teleport the player to the spawn location
    utils::teleport_player(&mut conn, config::get().spawn_location).await?;
    // Signal player skin layers to the client
    utils::signal_player_skin_layers(&mut conn, &player).await?;
    // Signal client to wait for level chunks
    utils::signal_game_state_change(&mut conn, EventType::WaitForLevelChunks, None).await?;
    // Signal center chunk to the client
    utils::signal_center_chunk(&mut conn).await?;

    let mut chunks_test = Vec::new();
    for x in -6..6 {
        for z in -6..6 {
            let mut section = Section::default();
            // plains biome, this should really be documented somewhere
            section.biomes.palette = azalea_world::palette::Palette::SingleValue(39);
            chunks_test.push((
                ChunkPos::new(x, z),
                Chunk {
                    sections: vec![section; 24],
                    heightmaps: Default::default(),
                },
            ));
        }
    }

    // Signal chunk batch update to the client
    utils::signal_chunk_batch_update(&mut conn, chunks_test).await?;

    // TODO: send chunks (it works, yay, now load them from the world)
    // TODO: keepalive (use conn.into_split() and move the writer to a separate task)
    // TODO: ...profit?

    // Player has fully joined the game at this point
    info!("Player has joined the game");

    loop {
        match conn.read_timeout(network::ConnectionPhase::Game).await {
            Ok(ServerboundGamePacket::Pong(_)) => {
                debug!("Received pong from client");
                break; // todo
            }
            Ok(_) => {} // todo
            Err(err) => bail_packet_error!(err, "Failed to read configuration packet"),
        }
    }

    Ok(())
}
