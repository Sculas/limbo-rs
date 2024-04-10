use std::{fmt::Debug, time::Duration};

use azalea_protocol::{
    connect::Connection,
    packets::{
        configuration::clientbound_disconnect_packet::ClientboundDisconnectPacket as ClientboundConfigurationDisconnectPacket,
        game::clientbound_disconnect_packet::ClientboundDisconnectPacket as ClientboundGameDisconnectPacket,
        login::clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket,
        ProtocolPacket,
    },
};
use tokio::io::AsyncWriteExt;
use tracing::*;

use super::{ConfigurationConnection, GameConnection, LoginConnection, Result};

pub static mut READ_TIMEOUT: Duration = Duration::from_secs(5);

#[macro_export]
macro_rules! timeout {
    ($e:expr) => {
        tokio::time::timeout(unsafe { $crate::network::ext::READ_TIMEOUT }, $e)
    };
}

pub trait ConnectionExt<R, W> {
    /// Read a packet from the other side of the connection with a timeout.
    async fn read_timeout(&mut self) -> Result<R>;
    /// Write a packet to the other side of the connection.
    async fn write_raw(&mut self, data: &[u8]) -> std::io::Result<()>;
}

impl<R, W> ConnectionExt<R, W> for Connection<R, W>
where
    R: ProtocolPacket + Debug,
    W: ProtocolPacket + Debug,
{
    async fn read_timeout(&mut self) -> Result<R> {
        Ok(tokio::time::timeout(unsafe { READ_TIMEOUT }, self.read()).await??)
    }

    async fn write_raw(&mut self, data: &[u8]) -> std::io::Result<()> {
        match self.writer.raw.write_stream.write_all(data).await {
            Ok(()) => Ok(()),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::BrokenPipe {
                    debug!("Broken pipe, shutting down connection.");
                    if let Err(e) = self.writer.shutdown().await {
                        error!("Couldn't shut down: {e}");
                    }
                }
                Err(err)
            }
        }
    }
}

pub trait ConnectionPhaseExt {
    async fn write_disconnect(&mut self, reason: String) -> std::io::Result<()>;
}

impl ConnectionPhaseExt for LoginConnection {
    async fn write_disconnect(&mut self, reason: String) -> std::io::Result<()> {
        self.write(
            ClientboundLoginDisconnectPacket {
                reason: reason.into(),
            }
            .get(),
        )
        .await
    }
}

impl ConnectionPhaseExt for ConfigurationConnection {
    async fn write_disconnect(&mut self, reason: String) -> std::io::Result<()> {
        self.write(
            ClientboundConfigurationDisconnectPacket {
                reason: reason.into(),
            }
            .get(),
        )
        .await
    }
}

impl ConnectionPhaseExt for GameConnection {
    async fn write_disconnect(&mut self, reason: String) -> std::io::Result<()> {
        self.write(
            ClientboundGameDisconnectPacket {
                reason: reason.into(),
            }
            .get(),
        )
        .await
    }
}
