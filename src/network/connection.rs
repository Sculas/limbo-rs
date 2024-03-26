use tracing::*;

use crate::{
    network::{
        self, phase,
        server::{AServer, Server},
        HandshakeConnection, NextPhase,
    },
    network_bail, player,
};

impl Server {
    #[tracing::instrument(name = "connection", skip_all, fields(addr))]
    pub async fn handle_connection(
        self: AServer,
        stream: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
    ) {
        let addr = player::addr::PlayerAddr::from(addr);
        tracing::Span::current().record("addr", tracing::field::display(addr));
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
    addr: player::addr::PlayerAddr,
    server: AServer,
) -> network::Result<()> {
    // Handle the handshake and transition to the next phase, if applicable.
    let mut conn = match phase::handshake::try_handle(&mut conn).await? {
        NextPhase::Status => return phase::status::try_handle(conn.status(), &server).await,
        NextPhase::Login => match phase::login::try_handle(conn.login(), addr, &server).await? {
            // Login phase continues to the next phase; transition to the configuration phase.
            NextPhase::Configuration(conn) => conn,
            phase => network_bail!("Unsupported next phase {phase:?} for login phase"),
        },
        phase => network_bail!("Unsupported next phase {phase:?} for handshake phase"),
    };
    Ok(())
}
