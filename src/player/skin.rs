use azalea_buf::McBufWritable;
pub use azalea_protocol::packets::configuration::serverbound_client_information_packet::ModelCustomization as SkinLayers;

pub const TEXTURE_KEY: &str = "textures";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSkin {
    // The texture of the player's skin.
    pub texture: String,
    // The signature of the player's skin.
    pub signature: String,
    // The visible layers of the player's skin.
    pub layers: SkinLayers,
}

pub trait SkinLayersExt {
    fn to_bits(&self) -> u8;
}

impl SkinLayersExt for SkinLayers {
    fn to_bits(&self) -> u8 {
        let mut set = azalea_core::bitset::FixedBitSet::<7>::new();
        if self.cape {
            set.set(0);
        }
        if self.jacket {
            set.set(1);
        }
        if self.left_sleeve {
            set.set(2);
        }
        if self.right_sleeve {
            set.set(3);
        }
        if self.left_pants {
            set.set(4);
        }
        if self.right_pants {
            set.set(5);
        }
        if self.hat {
            set.set(6);
        }

        let mut data = [0u8];
        set.write_into(&mut &mut data[..])
            .expect("BUG: Failed to write skin layers to bits");
        data[0]
    }
}
