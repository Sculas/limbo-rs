use azalea_protocol::packets::status::{
    clientbound_pong_response_packet::ClientboundPongResponsePacket,
    clientbound_status_response_packet::{ClientboundStatusResponsePacket, Players, Version},
    serverbound_ping_request_packet::ServerboundPingRequestPacket,
};
use tracing::*;

use crate::network::{self, StatusConnection};

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn respond_ping_req(
    conn: &mut StatusConnection,
    req: ServerboundPingRequestPacket,
) -> std::io::Result<()> {
    trace!("Responding to ping request");
    conn.write(ClientboundPongResponsePacket { time: req.time }.get())
        .await
}

#[tracing::instrument(level = "trace", skip(conn), err)]
pub async fn respond_status_ping(
    conn: &mut StatusConnection,
    version: &str,
    description: &str,
    online_players: i32,
    max_players: i32,
) -> std::io::Result<()> {
    trace!("Responding to status ping");
    conn.write(
        ClientboundStatusResponsePacket {
            description: description.into(),
            favicon: None,
            players: Players {
                max: max_players,
                online: online_players,
                sample: vec![],
            },
            version: Version {
                name: version.into(),
                protocol: network::PROTOCOL_VERSION,
            },
            enforces_secure_chat: Some(false),
        }
        .get(),
    )
    .await
}
