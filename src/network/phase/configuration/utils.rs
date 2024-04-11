use std::io::Cursor;

use azalea_buf::{McBufReadable, McBufWritable};
use azalea_core::resource_location::ResourceLocation;
use azalea_protocol::{
    connect::Connection,
    packets::configuration::{
        clientbound_custom_payload_packet::ClientboundCustomPayloadPacket,
        clientbound_finish_configuration_packet::ClientboundFinishConfigurationPacket,
        clientbound_registry_data_packet::ClientboundRegistryDataPacket,
    },
};
use tracing::*;

use crate::{
    config, internal_error,
    network::{self, server::PlayerRef, ConfigurationConnection, GameConnection},
    player::skin,
    utils::registry_data,
};

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn send_configurations(conn: &mut ConfigurationConnection) -> network::Result<()> {
    trace!("Sending server brand to client");
    send_server_brand(conn).await?;
    trace!("Sending registry data to client");
    conn.write(
        ClientboundRegistryDataPacket {
            registry_holder: registry_data::get(),
        }
        .get(),
    )
    .await?;
    trace!("Signaling client to finish configuration");
    conn.write(ClientboundFinishConfigurationPacket {}.get())
        .await?;
    Ok(())
}

async fn send_server_brand(conn: &mut ConfigurationConnection) -> std::io::Result<()> {
    let mut data = Vec::new();
    config::get().brand.write_into(&mut data)?;
    conn.write(
        ClientboundCustomPayloadPacket {
            identifier: ResourceLocation::new("minecraft:brand"),
            data: data.into(),
        }
        .get(),
    )
    .await
}

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn read_string(
    conn: &mut ConfigurationConnection,
    data: &[u8],
) -> network::Result<String> {
    trace!("Attempting to read string from packet data");
    match String::read_from(&mut Cursor::new(data)) {
        Ok(s) => Ok(s),
        Err(err) => internal_error!(conn, "Failed to read string from packet data: {err}"),
    }
}

#[tracing::instrument(level = "trace", skip_all)]
pub async fn set_skin_layers(player: &PlayerRef<'_>, layers: skin::SkinLayers) {
    trace!("Setting skin layers for player");
    player.lock().await.set_skin_layers(layers);
}

/// Change our state from configuration to game.
pub fn game(conn: ConfigurationConnection) -> GameConnection {
    Connection::from(conn)
}
