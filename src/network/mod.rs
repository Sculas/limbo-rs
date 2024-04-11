pub mod connection;
pub mod ext;
pub mod phase;
pub mod server;
pub mod utils;

const VERSION: &str = "1.20.4";
const PROTOCOL_VERSION: i32 = azalea_protocol::packets::PROTOCOL_VERSION;

pub type HandshakeConnection = azalea_protocol::connect::Connection<
    azalea_protocol::packets::handshaking::ServerboundHandshakePacket,
    azalea_protocol::packets::handshaking::ClientboundHandshakePacket,
>;

pub type StatusConnection = azalea_protocol::connect::Connection<
    azalea_protocol::packets::status::ServerboundStatusPacket,
    azalea_protocol::packets::status::ClientboundStatusPacket,
>;

pub type LoginConnection = azalea_protocol::connect::Connection<
    azalea_protocol::packets::login::ServerboundLoginPacket,
    azalea_protocol::packets::login::ClientboundLoginPacket,
>;

pub type ConfigurationConnection = azalea_protocol::connect::Connection<
    azalea_protocol::packets::configuration::ServerboundConfigurationPacket,
    azalea_protocol::packets::configuration::ClientboundConfigurationPacket,
>;

pub type GameConnection = azalea_protocol::connect::Connection<
    azalea_protocol::packets::game::ServerboundGamePacket,
    azalea_protocol::packets::game::ClientboundGamePacket,
>;

pub type Result<T> = std::result::Result<T, ConnectionError>;

#[derive(thiserror::Error, Debug)]
pub enum ConnectionError {
    // server imposed disconnect
    #[error("client disconnected: {0}")]
    Disconnect(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("error while reading packet: {0}")]
    ReadPacket(#[from] Box<azalea_protocol::read::ReadPacketError>),
    #[error("read timeout while reading packet in phase {0:?}")]
    ReadTimeout(ConnectionPhase),
}

impl ConnectionError {
    /// Returns `true` if the connection was closed by the client.
    /// Returns `false` otherwise.
    pub fn connection_closed(&self) -> bool {
        match self {
            ConnectionError::ReadPacket(err) => matches!(
                **err,
                azalea_protocol::read::ReadPacketError::ConnectionClosed
            ),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum ClientIntention {
    Status,
    Login,
}

#[derive(Debug)]
pub enum ConnectionPhase {
    Handshake,
    Status,
    Login,
    Configuration,
    Game,
}
