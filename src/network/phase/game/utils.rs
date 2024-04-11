use azalea_core::{
    game_type::{GameMode, OptionalGameType},
    resource_location::ResourceLocation,
};
use azalea_protocol::packets::{
    common::CommonPlayerSpawnInfo, game::clientbound_login_packet::ClientboundLoginPacket,
};
use tracing::*;

use crate::{
    config,
    network::{
        self,
        server::{constants, AServer, PlayerRef},
        GameConnection,
    },
};

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn signal_game_start(
    conn: &mut GameConnection,
    server: &AServer,
    player: &PlayerRef,
) -> network::Result<()> {
    trace!("Signaling game start to client");
    let player_id = player.lock().await.entity_id();
    let max_players = config::get().max_players;
    conn.write(
        ClientboundLoginPacket {
            player_id,
            hardcore: false,
            levels: Vec::new(), // todo
            max_players,
            chunk_radius: constants::VIEW_DISTANCE,
            simulation_distance: constants::SIMULATION_DISTANCE,
            reduced_debug_info: !cfg!(debug_assertions),
            show_death_screen: true,
            do_limited_crafting: false,
            common: CommonPlayerSpawnInfo {
                dimension_type: ResourceLocation::new("minecraft:overworld"),
                dimension: ResourceLocation::new("minecraft:world"),
                seed: 0,
                game_type: GameMode::Adventure,
                previous_game_type: OptionalGameType(None),
                is_debug: false,
                is_flat: true,
                last_death_location: None,
                portal_cooldown: 0,
            },
        }
        .get(),
    )
    .await?;
    Ok(())
}
