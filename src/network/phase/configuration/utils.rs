use std::io::Cursor;

use azalea_buf::McBufReadable;
use azalea_protocol::connect::Connection;
use tracing::*;

use crate::{
    internal_error,
    network::{self, ConfigurationConnection, GameConnection},
};

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn read_string(
    conn: &mut ConfigurationConnection,
    data: &[u8],
) -> network::Result<String> {
    trace!("Attempting to read string from packet data");
    match String::read_from(&mut Cursor::new(data)) {
        Ok(s) => Ok(s),
        Err(err) => internal_error!(conn, "Failed to read string from packet data: {err}"),
    }
}

/// Change our state from configuration to game.
pub fn game(conn: ConfigurationConnection) -> GameConnection {
    Connection::from(conn)
}
