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
    #[error("client disconnected: {0}")]
    Disconnect(String),
    #[error("error while reading packet: {0}")]
    ReadPacket(#[from] Box<azalea_protocol::read::ReadPacketError>),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("read timeout while reading packet: {0}")]
    ReadTimeout(#[from] tokio::time::error::Elapsed),
}

pub enum NextPhase {
    Status,
    Login,
    Configuration(ConfigurationConnection),
    Game,
}

impl std::fmt::Debug for NextPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NextPhase::Status => write!(f, "Status"),
            NextPhase::Login => write!(f, "Login"),
            NextPhase::Configuration(_) => write!(f, "Configuration"),
            NextPhase::Game => write!(f, "Game"),
        }
    }
}
