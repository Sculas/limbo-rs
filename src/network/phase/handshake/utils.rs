use azalea_protocol::packets::handshaking::{
    client_intention_packet::ClientIntentionPacket, ServerboundHandshakePacket,
};
use tracing::*;

use crate::{
    bail_packet_error,
    network::{self, ext::ConnectionExt, HandshakeConnection},
    timeout,
};

// Legacy ping constants
const SEP: char = '\0';
const PROTOCOL: i8 = 127;

#[tracing::instrument(level = "trace", skip(conn), ret, err)]
pub async fn is_legacy_ping(conn: &mut HandshakeConnection) -> network::Result<bool> {
    trace!("Checking for legacy ping");
    let buf = &mut [0u8; 1];
    let n = timeout!(
        conn.reader.raw.read_stream.peek(buf),
        network::ConnectionPhase::Handshake
    )
    .await??;
    Ok(n == 1 && buf[0] == 0xFE)
}

#[tracing::instrument(level = "trace", skip(conn), err)]
pub async fn respond_legacy_ping(
    conn: &mut HandshakeConnection,
    version: &str,
    description: &str,
    online_players: i32,
    max_players: i32,
) -> network::Result<()> {
    trace!("Responding to legacy ping");
    let mut buf = Vec::new();
    // packet ID and length placeholder
    buf.extend([0xff, 0x00, 0x00]);
    // legacy ping constant
    buf.extend("§1\0".encode_utf16().flat_map(|c| c.to_be_bytes()));
    // legacy ping response
    buf.extend(
        format!(
            "{PROTOCOL}{SEP}{version}{SEP}{description}{SEP}{online_players}{SEP}{max_players}"
        )
        .encode_utf16()
        .flat_map(|c| c.to_be_bytes()),
    );
    // replace the length placeholder with the actual length
    let chars = (buf.len() as u16 - 3) / 2;
    buf[1..3].copy_from_slice(chars.to_be_bytes().as_slice());
    // send the response
    conn.write_raw(&buf).await?;
    Ok(())
}

#[tracing::instrument(level = "trace", skip(conn), ret, err)]
pub async fn read_intent(conn: &mut HandshakeConnection) -> network::Result<ClientIntentionPacket> {
    trace!("Reading client intent");
    match conn.read_timeout(network::ConnectionPhase::Handshake).await {
        Ok(ServerboundHandshakePacket::ClientIntention(packet)) => Ok(packet),
        Err(err) => bail_packet_error!(err, "Failed to read client intention"),
    }
}
