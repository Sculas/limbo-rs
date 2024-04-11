use azalea_protocol::packets::configuration::ServerboundConfigurationPacket;
use tracing::*;

use crate::{
    bail_packet_error,
    network::{self, ext::ConnectionExt, server::PlayerRef},
};

mod utils;

/// Attempts to handle the configuration phase.
#[tracing::instrument(name = "configuration", skip_all)]
pub async fn try_handle(
    mut conn: network::ConfigurationConnection,
    player: &PlayerRef<'_>,
) -> network::Result<network::GameConnection> {
    debug!("Handling configuration phase");
    let mut brand_received = false;

    // Send configurations to the client
    utils::send_configurations(&mut conn).await?;

    loop {
        match conn
            .read_timeout(network::ConnectionPhase::Configuration)
            .await
        {
            Ok(ServerboundConfigurationPacket::ClientInformation(packet)) => {
                debug!("Received client information from client");
                utils::set_skin_layers(player, packet.information.model_customization).await;
            }
            Ok(ServerboundConfigurationPacket::CustomPayload(packet)) => {
                debug!("Received custom payload from client");

                // Handle the custom payload
                match packet.identifier.to_string().as_str() {
                    "minecraft:brand" => {
                        // Since we log the brand, we don't want the client to spam it
                        if brand_received {
                            continue;
                        }
                        brand_received = true;

                        let brand = utils::read_string(&mut conn, &packet.data[..]).await?;
                        info!(%brand, "Client brand received");
                    }
                    _ => {} // we don't care about other custom payloads
                }
            }
            Ok(ServerboundConfigurationPacket::KeepAlive(_)) => {
                debug!("Received keep alive from client");
            }
            Ok(ServerboundConfigurationPacket::Pong(_)) => {
                debug!("Received pong from client");
            }
            Ok(ServerboundConfigurationPacket::ResourcePack(_)) => {
                debug!("Received resource pack from client");
            }
            Ok(ServerboundConfigurationPacket::FinishConfiguration(_)) => {
                debug!("Received finish configuration from client");
                break;
            }
            Err(err) => bail_packet_error!(err, "Failed to read configuration packet"),
        }
    }

    debug!("Transitioning to game phase");
    Ok(utils::game(conn))
}
