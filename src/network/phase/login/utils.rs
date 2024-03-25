use azalea_core::resource_location::ResourceLocation;
use azalea_protocol::{
    connect::Connection,
    packets::login::{
        clientbound_custom_query_packet::ClientboundCustomQueryPacket,
        clientbound_game_profile_packet::ClientboundGameProfilePacket,
    },
};
use tracing::*;

use crate::{
    config, internal_error,
    network::{self, server::AServer, ConfigurationConnection, LoginConnection},
    network_disconnect,
    player::{Player, PlayerBuilder},
    utils::velocity,
};

/// Change our state from login to configuration.
pub fn configuration(conn: LoginConnection) -> ConfigurationConnection {
    Connection::from(conn)
}

#[tracing::instrument(level = "trace", skip(conn), err)]
pub async fn notify_velocity_forwarding(
    conn: &mut LoginConnection,
    transaction_id: u32,
) -> std::io::Result<()> {
    trace!("Notifying Velocity of client login");
    conn.write(
        ClientboundCustomQueryPacket {
            transaction_id,
            identifier: ResourceLocation::new(velocity::VELOCITY_CHANNEL),
            data: Vec::from(velocity::FORWARDING_VERSION_BYTES).into(),
        }
        .get(),
    )
    .await
}

#[tracing::instrument(level = "trace", skip(conn), err)]
pub async fn verify_velocity_forwarding(
    conn: &mut LoginConnection,
    want_tid: u32,
    have_tid: u32,
    data: Option<&[u8]>,
) -> network::Result<velocity::ForwardingInfo> {
    trace!("Verifying Velocity forwarding secret");
    if want_tid != have_tid {
        network_disconnect!(
            conn,
            "Transaction ID mismatch: expected {want_tid}, got {have_tid}"
        );
    }

    let config = config::get();
    debug_assert!(
        config.uses_velocity_modern,
        "Velocity forwarding verification called without modern forwarding enabled"
    );
    let secret = config.velocity_forwarding_secret.as_bytes();
    match data {
        Some(data) => match velocity::verify_forwarding_info(data, secret) {
            Ok(info) => {
                debug!("Verified Velocity forwarding: {info:?}");
                Ok(info)
            }
            Err(err) => network_disconnect!(conn, "Failed to verify Velocity forwarding: {err}"),
        },
        None => network_disconnect!(conn, "Invalid Velocity response data"),
    }
}

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn build_player(
    conn: &mut LoginConnection,
    player: &PlayerBuilder,
) -> network::Result<Player> {
    trace!("Attempting to build player data");
    match player.build() {
        Ok(player) => Ok(player),
        Err(err) => {
            error!("Failed to build player data: {err}");
            internal_error!(conn, "Failed to build player data")
        }
    }
}

#[tracing::instrument(level = "trace", skip_all, fields(%player), err)]
pub async fn signal_login_success(
    conn: &mut LoginConnection,
    server: &AServer,
    player: Player,
) -> std::io::Result<()> {
    trace!("Sending login success");
    server.add_player(player.clone()).await;
    conn.write(
        ClientboundGameProfilePacket {
            game_profile: player.into(),
        }
        .get(),
    )
    .await
}
