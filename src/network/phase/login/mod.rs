use azalea_protocol::packets::login::ServerboundLoginPacket;
use rand::Rng;
use tracing::*;

use crate::{
    bail_packet_error, config,
    network::{self, ext::ConnectionExt, server::AServer, NextPhase},
    network_disconnect, network_state,
    player::PlayerBuilder,
};

mod utils;

network_state! {
    Hello,
    QueryAnswer,
    PhaseSwitch,
}

/// Attempts to handle an status ping.
#[tracing::instrument(name = "login", skip_all)]
pub async fn try_handle(
    mut conn: network::LoginConnection,
    addr: std::net::SocketAddr,
    server: &AServer,
) -> network::Result<NextPhase> {
    debug!("Handling login phase");
    let mut state = State::default();

    let config = config::get();
    let transaction_id = rand::thread_rng().gen();
    let mut player = PlayerBuilder::default();
    player.addr(addr); // set player IP address

    loop {
        match conn.read_timeout().await {
            Ok(ServerboundLoginPacket::Hello(packet)) => {
                debug!(username = ?packet.name, uuid = ?packet.profile_id, "Received hello from client");
                validate_state!(conn, state == State::Hello, "Unexpected hello packet");

                if config.uses_velocity_modern {
                    utils::notify_velocity_forwarding(&mut conn, transaction_id).await?;
                    state = State::QueryAnswer;
                    continue; // await verification from Velocity
                }

                let offline_uuid = azalea_auth::offline::generate_uuid(&packet.name);
                player.name(packet.name).uuid(offline_uuid);
                let player_data = utils::build_player(&mut conn, &player).await?;
                utils::signal_login_success(&mut conn, server, player_data).await?;
                state = State::PhaseSwitch; // wait for login ack before transitioning
            }
            Ok(ServerboundLoginPacket::Key(_)) => {
                warn!("Received unexpected encryption response from client");
                network_disconnect!(&mut conn, "Unexpected packet received");
            }
            Ok(ServerboundLoginPacket::CustomQueryAnswer(packet)) => {
                debug!(
                    transaction_id = packet.transaction_id,
                    data = packet.data.as_ref().map(|d| format!("{} bytes", d.len())),
                    "Received custom query answer from client"
                );
                validate_state!(
                    conn,
                    state == State::QueryAnswer,
                    "Unexpected custom query answer packet"
                );

                if config.uses_velocity_modern {
                    let info = utils::verify_velocity_forwarding(
                        &mut conn,
                        transaction_id,
                        packet.transaction_id,
                        packet.data.as_deref(),
                    )
                    .await?;

                    #[rustfmt::skip] // keep it on a single line
                    player.addr(info.addr).name(info.name).uuid(info.uuid).skin(info.skin);
                    let player_data = utils::build_player(&mut conn, &player).await?;
                    utils::signal_login_success(&mut conn, server, player_data).await?;
                    state = State::PhaseSwitch; // wait for login ack before transitioning
                }
            }
            Ok(ServerboundLoginPacket::LoginAcknowledged(_)) => {
                debug!("Received login acknowledgement from client");
                validate_state!(
                    conn,
                    state == State::PhaseSwitch,
                    "Unexpected login acknowledgement packet"
                );
                break; // next phase: configuration
            }
            Err(err) => bail_packet_error!(err, "Failed to read login packet"),
        }
    }

    Ok(NextPhase::Configuration(utils::configuration(conn)))
}