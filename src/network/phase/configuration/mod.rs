use azalea_protocol::packets::configuration::{
    clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket,
    clientbound_registry_data_packet::ClientboundRegistryDataPacket,
    ServerboundConfigurationPacket,
};
use tracing::*;

use crate::{
    bail_packet_error,
    network::{self, ext::ConnectionExt},
    utils::registry_data,
};

mod utils;

/// Attempts to handle the configuration phase.
#[tracing::instrument(name = "configuration", skip_all)]
pub async fn try_handle(
    mut conn: network::ConfigurationConnection,
) -> network::Result<network::GameConnection> {
    debug!("Handling configuration phase");
    let mut brand_received = false;

    conn.write(
        ClientboundRegistryDataPacket {
            registry_holder: registry_data::get(),
        }
        .get(),
    )
    .await?;
    conn.write(ClientboundFinishConfigurationPacket {}.get())
        .await?;

    loop {
        match conn.read_timeout().await {
            Ok(ServerboundConfigurationPacket::ClientInformation(_)) => {
                debug!("Received client information from client");
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