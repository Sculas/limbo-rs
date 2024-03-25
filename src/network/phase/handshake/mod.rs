use azalea_protocol::packets::ConnectionProtocol;
use tracing::*;

use crate::{
    network::{self, NextPhase},
    network_bail,
};

mod utils;

/// Attempts to handle an incoming handshake.
#[tracing::instrument(name = "handshake", skip_all)]
pub async fn try_handle(conn: &mut network::HandshakeConnection) -> network::Result<NextPhase> {
    debug!("Handling incoming handshake");

    // Handle legacy ping (<=1.6)
    if utils::is_legacy_ping(conn).await? {
        debug!("Legacy ping detected");
        utils::respond_legacy_ping(
            conn,
            network::VERSION,
            &format!(
                "Unsupported client version. Please use {} instead.",
                network::VERSION
            ),
            0, // online players
            0, // max players
        )
        .await?;
        return Err(network::ConnectionError::Disconnect(
            "Unsupported client version".into(),
        ));
    }

    // Handle >=1.7 handshake
    let intent = utils::read_intent(conn).await?;
    debug!(version = intent.protocol_version, host = intent.hostname, port = intent.port, intention = ?intent.intention, "Received client intention");
    match intent.intention {
        ConnectionProtocol::Status => Ok(NextPhase::Status),
        ConnectionProtocol::Login => Ok(NextPhase::Login),
        intention => {
            warn!(intention = ?intention, phase = "handshake", "Unsupported client intention at current phase");
            network_bail!("Unsupported client intention");
        }
    }
}
