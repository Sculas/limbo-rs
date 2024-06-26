use azalea_auth::game_profile::GameProfile;

use crate::network::server::AServer;

pub mod skin;

pub struct Player {
    // Player Data
    /// The IP address of the player.
    addr: std::net::SocketAddr,
    /// The username of the player.
    name: String,
    /// The UUID of the player.
    uuid: uuid::Uuid,
    /// The skin of the player.
    skin: Option<skin::PlayerSkin>,
    // Entity Data
    /// The entity ID of the player.
    entity_id: u32,
}

impl Player {
    /// Creates a new player that must be initialized.
    pub fn new(
        addr: std::net::SocketAddr,
        name: String,
        uuid: uuid::Uuid,
        skin: Option<skin::PlayerSkin>,
    ) -> Self {
        Self {
            addr,
            name,
            uuid,
            skin,
            entity_id: 0,
        }
    }

    /// Initializes the player.
    /// Must be called once the player enters the game phase.
    pub fn init(&mut self, server: &AServer) {
        self.entity_id = server.next_entity_id();
    }

    /// Returns the IP address of the player.
    pub fn addr(&self) -> &std::net::SocketAddr {
        &self.addr
    }

    /// Returns the username of the player.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the UUID of the player.
    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    /// Returns the skin of the player.
    pub fn skin(&self) -> Option<&skin::PlayerSkin> {
        self.skin.as_ref()
    }

    /// Returns the entity ID of the player.
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Returns a clone of the player's game profile (including skin data).
    pub fn game_profile(&self) -> GameProfile {
        let mut profile = azalea_auth::game_profile::GameProfile::new(self.uuid, self.name.clone());
        if let Some(skin) = self.skin.clone() {
            profile.properties.insert(
                skin::TEXTURE_KEY.into(),
                azalea_auth::game_profile::ProfilePropertyValue {
                    value: skin.texture,
                    signature: Some(skin.signature),
                },
            );
        }
        profile
    }

    /// Sets the visible layers of the player's skin.
    pub fn set_skin_layers(&mut self, layers: skin::SkinLayers) {
        if let Some(skin) = self.skin.as_mut() {
            skin.layers = layers;
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player({} / {})", self.name, self.uuid)
    }
}
