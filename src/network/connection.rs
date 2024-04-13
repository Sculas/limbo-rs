use tracing::*;

use crate::{
    config,
    network::{
        self, phase,
        server::{AServer, Server},
        ClientIntention, ConnectionError, HandshakeConnection,
    },
};

impl Server {
    #[tracing::instrument(name = "connection", skip_all, fields(addr, player))]
    pub async fn handle_connection(
        self: AServer,
        stream: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
    ) {
        if !config::get().hide_player_ips {
            tracing::Span::current().record("addr", tracing::field::display(addr));
        }

        debug!("Handling incoming connection");
        match try_handle(HandshakeConnection::wrap(stream), addr, self).await {
            Ok(()) => debug!("Connection closed by server"),
            Err(err) => match err {
                ConnectionError::Disconnect(reason) => debug!("Player disconnected: {reason}"),
                err if err.connection_closed() => debug!("Connection closed by client"),
                err => error!("Error while handling connection: {err}"),
            },
        }
    }
}

async fn try_handle(
    mut conn: HandshakeConnection,
    addr: std::net::SocketAddr,
    server: AServer,
) -> network::Result<()> {
    // Handle the handshake and transition to the configuration next phase, if applicable.
    let (conn, ref player) = match phase::handshake::try_handle(&mut conn).await? {
        ClientIntention::Status => return phase::status::try_handle(conn.status(), &server).await,
        ClientIntention::Login => phase::login::try_handle(conn.login(), addr, &server).await?,
    };
    let player_record = format!("{}", player.lock().await);
    tracing::Span::current().record("player", tracing::field::display(player_record));
    let conn = phase::configuration::try_handle(conn, player).await?;
    phase::game::try_handle(conn, &server, player).await // no further phases, we've reached the gameloop
}
