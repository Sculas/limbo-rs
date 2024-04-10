use tracing::*;

use crate::{
    network::{
        self, phase,
        server::{AServer, Server},
        ClientIntention, HandshakeConnection,
    },
    player,
};

// TODO: add Player to connection span

impl Server {
    #[tracing::instrument(name = "connection", skip_all, fields(addr, player))]
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
    // Handle the handshake and transition to the configuration next phase, if applicable.
    let (conn, player) = match phase::handshake::try_handle(&mut conn).await? {
        ClientIntention::Status => return phase::status::try_handle(conn.status(), &server).await,
        ClientIntention::Login => phase::login::try_handle(conn.login(), addr, &server).await?,
    };
    let conn = phase::configuration::try_handle(conn).await?;
    Ok(())
}
