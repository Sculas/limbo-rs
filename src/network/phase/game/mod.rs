use azalea_protocol::packets::game::ServerboundGamePacket;
use tracing::*;

use crate::{
    bail_packet_error,
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

    // Signal game start to the client
    utils::signal_game_start(&mut conn, server, &player).await?;

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
