pub const TEXTURE_KEY: &str = "textures";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSkin {
    // The texture of the player's skin.
    pub texture: String,
    // The signature of the player's skin.
    pub signature: String,
}
