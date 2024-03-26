pub mod addr;
pub mod skin;

#[derive(derive_builder::Builder, Clone, Debug, PartialEq, Eq)]
pub struct Player {
    // The IP address of the player.
    pub addr: addr::PlayerAddr,
    // The username of the player.
    pub name: String,
    // The UUID of the player.
    pub uuid: uuid::Uuid,
    // The skin of the player.
    #[builder(default)]
    pub skin: Option<skin::PlayerSkin>,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player({}; {})", self.name, self.uuid)
    }
}

impl From<Player> for azalea_auth::game_profile::GameProfile {
    fn from(player: Player) -> Self {
        let mut p = azalea_auth::game_profile::GameProfile::new(player.uuid, player.name);
        if let Some(skin) = player.skin {
            p.properties.insert(
                skin::TEXTURE_KEY.into(),
                azalea_auth::game_profile::ProfilePropertyValue {
                    value: skin.texture,
                    signature: Some(skin.signature),
                },
            );
        }
        p
    }
}
