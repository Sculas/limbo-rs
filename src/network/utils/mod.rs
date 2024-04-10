pub mod state;

#[macro_export]
macro_rules! bail_packet_error {
    ($err:ident, $ctx:expr) => {{
        return Err($err);
    }};
}

// Logs a message and disconnects the client immediately without sending a packet.
#[macro_export]
macro_rules! network_bail {
    ($($reason:tt)+) => {{
        let reason = format!($($reason)+);
        tracing::warn!("Disconnecting client with reason: {}", reason);
        return Err($crate::network::ConnectionError::Disconnect(reason));
    }};
}

#[macro_export]
macro_rules! network_disconnect {
    ($conn:ident, $($reason:tt)+) => {{
        let reason = format!($($reason)+);
        $crate::network::ext::ConnectionPhaseExt::write_disconnect($conn, reason.clone()).await?;
        $crate::network_bail!("{}", reason);
    }};
    (&mut $conn:ident, $($reason:tt)+) => {{
        let reason = format!($($reason)+);
        $crate::network::ext::ConnectionPhaseExt::write_disconnect(&mut $conn, reason.clone()).await?;
        $crate::network_bail!("{}", reason);
    }};
}

#[macro_export]
macro_rules! internal_error {
    ($conn:ident, $($reason:tt)+) => {{
        let reason = format!($($reason)+);
        tracing::error!("An internal error occurred: {}", reason);
        $crate::network_disconnect!($conn, "Internal error: {}", reason);
    }};
}
