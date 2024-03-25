use std::net::SocketAddr;

use tracing::*;

use crate::{
    network::{
        self, phase,
        server::{AServer, Server},
        HandshakeConnection, NextPhase,
    },
    network_bail,
};

impl Server {
    #[tracing::instrument(name = "connection", skip(self, stream))]
    pub async fn handle_connection(self: AServer, stream: tokio::net::TcpStream, addr: SocketAddr) {
        debug!("Handling incoming connection");
        match try_handle(HandshakeConnection::wrap(stream), addr, self).await {
            Ok(()) => debug!("Connection closed successfully"),
            Err(network::ConnectionError::Disconnect(reason)) => {
                debug!("Connection closed: {reason}")
            }
            Err(err) => error!("Error while handling connection: {err:?}"),
        }
    }
}

async fn try_handle(
    mut conn: HandshakeConnection,
    addr: SocketAddr,
    server: AServer,
) -> network::Result<()> {
    // Handle the handshake and transition to the next phase, if applicable.
    let mut conn = match phase::handshake::try_handle(&mut conn).await? {
        NextPhase::Status => return phase::status::try_handle(conn.status(), &server).await,
        NextPhase::Login => match phase::login::try_handle(conn.login(), addr, &server).await? {
            // Login phase continues to the next phase; transition to the configuration phase.
            NextPhase::Configuration(conn) => conn,
            phase => network_bail!("Unsupported next phase {phase:?}"),
        },
        phase => network_bail!("Unsupported next phase {phase:?}"),
    };
    Ok(())
}
