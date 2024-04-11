use azalea_protocol::packets::login::ServerboundLoginPacket;
use rand::Rng;
use tracing::*;

use crate::{
    bail_packet_error, config,
    network::{
        self,
        ext::ConnectionExt,
        server::{AServer, PlayerRef},
    },
    network_disconnect, network_state,
    player::{addr::PlayerAddr, Player},
};

mod utils;

network_state! { phase => "Login";
    Hello,
    QueryAnswer,
    PhaseSwitch,
}

/// Attempts to handle the login phase.
#[tracing::instrument(name = "login", skip_all)]
pub async fn try_handle(
    mut conn: network::LoginConnection,
    addr: PlayerAddr,
    server: &AServer,
) -> network::Result<(network::ConfigurationConnection, PlayerRef)> {
    debug!("Handling login phase");
    let mut state = State::default();

    let config = config::get();
    let transaction_id = rand::thread_rng().gen();
    let mut player_lock = None;

    loop {
        match conn.read_timeout(network::ConnectionPhase::Login).await {
            Ok(ServerboundLoginPacket::Hello(packet)) => {
                debug!(username = ?packet.name, uuid = ?packet.profile_id, "Received hello from client");
                validate_state!(conn, state == State::Hello, "Unexpected hello packet");

                if config.uses_velocity_modern {
                    utils::notify_velocity_forwarding(&mut conn, transaction_id).await?;
                    state = State::QueryAnswer;
                    continue; // await verification from Velocity
                }

                // TODO: handle illegal names
                let player = Player::new(
                    addr,
                    packet.name.clone(),
                    azalea_auth::offline::generate_uuid(&packet.name),
                    None,
                );
                player_lock = Some(utils::signal_login_success(&mut conn, server, player).await?);
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

                    let player = Player::new(info.addr.into(), info.name, info.uuid, info.skin);
                    player_lock =
                        Some(utils::signal_login_success(&mut conn, server, player).await?);
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

    debug!("Transitioning to configuration phase");
    Ok((utils::configuration(conn), player_lock.unwrap()))
}
