use azalea_protocol::packets::status::ServerboundStatusPacket;
use tracing::*;

use crate::{
    bail_packet_error, config,
    network::{self, ext::ConnectionExt, server::AServer},
};

mod utils;

/// Attempts to handle an status ping.
#[tracing::instrument(name = "status", skip_all)]
pub async fn try_handle(
    mut conn: network::StatusConnection,
    server: &AServer,
) -> network::Result<()> {
    debug!("Handling status ping");

    let config = config::get();
    loop {
        match conn.read_timeout().await {
            Ok(ServerboundStatusPacket::StatusRequest(_)) => {
                let player_count = server.get_player_count();
                utils::respond_status_ping(
                    &mut conn,
                    &config.version,
                    &config.motd,
                    player_count as i32,
                    config.max_players,
                )
                .await?
                // wait for ping request, if client wishes to do so
            }
            Ok(ServerboundStatusPacket::PingRequest(req)) => {
                utils::respond_ping_req(&mut conn, req).await?;
                break; // close the connection
            }
            // The client may disconnect after StatusResponse (while we're still waiting for PingRequest),
            // so we should just break out of the loop and close the connection in that case.
            Err(err) if err.connection_closed() => break,
            Err(err) => bail_packet_error!(err, "Failed to read status packet"),
        }
    }

    Ok(())
}
