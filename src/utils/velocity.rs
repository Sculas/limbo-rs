use crate::player::skin;
use azalea_buf::{McBufReadable, McBufVarReadable};

pub const VELOCITY_CHANNEL: &str = "velocity:player_info";

// Corresponds to MODERN_DEFAULT forwarding mode:
// https://github.com/PaperMC/Velocity/blob/dev/3.0.0/proxy/src/main/java/com/velocitypowered/proxy/connection/PlayerDataForwarding.java#L44
pub const FORWARDING_VERSION: u8 = 1;
pub const FORWARDING_VERSION_BYTES: &[u8] = &[FORWARDING_VERSION];

#[derive(Debug)]
pub struct ForwardingInfo {
    // The actual IP address of the player.
    pub addr: std::net::SocketAddr,
    // The username of the player.
    pub name: String,
    // The UUID of the player.
    pub uuid: uuid::Uuid,
    // The skin of the player.
    pub skin: Option<skin::PlayerSkin>,
}

#[derive(thiserror::Error, Debug)]
pub enum VelocityError {
    #[error("invalid signature length")]
    InvalidSignatureLength,
    #[error("invalid signature provided")]
    InvalidSignature,
    #[error("unsupported version: {0} (expected {FORWARDING_VERSION})")]
    UnsupportedVersion(u8),
    #[error("failed to parse addr: {0}")]
    AddrParse(#[from] std::net::AddrParseError),
    #[error("read error: {0}")]
    Read(#[from] azalea_buf::BufReadError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn verify_forwarding_info(data: &[u8], secret: &[u8]) -> Result<ForwardingInfo, VelocityError> {
    let mut data = std::io::Cursor::new(verify(data, secret)?);
    let version: i32 = McBufVarReadable::var_read_from(&mut data)?;
    if version as u8 != FORWARDING_VERSION {
        return Err(VelocityError::UnsupportedVersion(version as u8));
    }
    let addr: String = McBufReadable::read_from(&mut data)?;
    let uuid: uuid::Uuid = McBufReadable::read_from(&mut data)?;
    let name: String = McBufReadable::read_from(&mut data)?;
    let skin = find_skin_data(&mut data)?;
    Ok(ForwardingInfo {
        addr: addr.parse()?,
        name,
        uuid,
        skin,
    })
}

fn verify<'a>(data: &'a [u8], secret: &'a [u8]) -> Result<&'a [u8], VelocityError> {
    if data.len() <= 32 {
        return Err(VelocityError::InvalidSignatureLength);
    }

    let (signature, data) = data.split_at(32);
    let calc_sig = hmac_sha256::HMAC::mac(data, secret);
    if signature != calc_sig.as_slice() {
        return Err(VelocityError::InvalidSignature);
    }

    Ok(data)
}

fn find_skin_data(
    data: &mut std::io::Cursor<&[u8]>,
) -> Result<Option<skin::PlayerSkin>, VelocityError> {
    let len: i32 = McBufVarReadable::var_read_from(data)?;
    for _ in 0..len {
        let name: String = McBufReadable::read_from(data)?;
        let value: String = McBufReadable::read_from(data)?;
        let mut maybe_sig: Option<String> = None;
        let has_sig: bool = McBufReadable::read_from(data)?;
        if has_sig {
            maybe_sig = Some(McBufReadable::read_from(data)?);
        }

        if name == skin::TEXTURE_KEY {
            return Ok(Some(skin::PlayerSkin {
                texture: value,
                signature: maybe_sig.unwrap_or_default(),
            }));
        }
    }
    Ok(None)
}
