use std::io::Cursor;

use azalea_buf::McBufReadable;
use azalea_core::registry_holder::RegistryHolder;
use once_cell::sync::Lazy;

const REGISTRY_BYTES: &[u8] = include_bytes!("dimension_registry.nbt");
const REGISTRY_DATA: Lazy<RegistryHolder> = Lazy::new(|| {
    RegistryHolder::read_from(&mut Cursor::new(REGISTRY_BYTES))
        .expect("Failed to read registry data")
});

/// Returns a clone of the registry data.
pub fn get() -> RegistryHolder {
    REGISTRY_DATA.clone()
}
