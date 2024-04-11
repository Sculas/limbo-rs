use once_cell::sync::Lazy;

static ADDR_WITHHELD: Lazy<bool> = Lazy::new(|| crate::config::get().hide_player_ips);

// The IP address of a player.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerAddr(std::net::SocketAddr);

impl std::fmt::Display for PlayerAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *ADDR_WITHHELD {
            write!(f, "<redacted>")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl std::fmt::Debug for PlayerAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl From<std::net::SocketAddr> for PlayerAddr {
    fn from(addr: std::net::SocketAddr) -> Self {
        Self(addr)
    }
}

impl AsRef<std::net::SocketAddr> for PlayerAddr {
    fn as_ref(&self) -> &std::net::SocketAddr {
        &self.0
    }
}

impl std::ops::Deref for PlayerAddr {
    type Target = std::net::SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
