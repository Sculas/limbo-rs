use std::io::Cursor;

use anyhow::Context;
use azalea_core::registry_holder::RegistryHolder;
use tracing::*;

// A very big thank you to Norbiros for dumping this packet:
// https://gist.github.com/Norbiros/f604ce46821e68c50260a169a9921560
// NOTE: This MUST be NBT and NOT Network NBT! Use net2nbt to convert.
const REGISTRY_BYTES: &[u8] = include_bytes!("dimension_registry.nbt");
static mut REGISTRY_DATA: Option<RegistryHolder> = None;

pub fn init() -> anyhow::Result<()> {
    info!("Initializing registry data");

    let nbt = simdnbt::borrow::Nbt::read(&mut Cursor::new(REGISTRY_BYTES))
        .context("Failed to parse registry data")?;
    let holder = RegistryHolder {
        map: simdnbt::Deserialize::from_compound(&nbt.unwrap())?,
    };
    if holder.map.is_empty() {
        anyhow::bail!("Corrupted registry data (empty)");
    }

    unsafe { REGISTRY_DATA = Some(holder) };
    Ok(())
}

/// Returns a clone of the registry data.
pub fn get() -> RegistryHolder {
    unsafe {
        REGISTRY_DATA
            .clone()
            .expect("Registry data not initialized")
    }
}
