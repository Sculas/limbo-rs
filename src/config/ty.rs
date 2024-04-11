use azalea_core::game_type::GameMode as AGameMode;

pub struct GameMode(AGameMode);

impl serde::Serialize for GameMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.short_name().to_lowercase())
    }
}

impl<'de> serde::Deserialize<'de> for GameMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        match name.to_lowercase().as_str() {
            "survival" => Ok(GameMode(AGameMode::Survival)),
            "creative" => Ok(GameMode(AGameMode::Creative)),
            "adventure" => Ok(GameMode(AGameMode::Adventure)),
            "spectator" => Ok(GameMode(AGameMode::Spectator)),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown game type name: {name}"
            ))),
        }
    }
}

impl std::ops::Deref for GameMode {
    type Target = AGameMode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
