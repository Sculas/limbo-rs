use azalea_core::{
    game_type::{GameMode, OptionalGameType},
    resource_location::ResourceLocation,
};
use azalea_entity::{EntityDataItem, EntityDataValue, EntityMetadataItems};
use azalea_protocol::packets::{
    common::CommonPlayerSpawnInfo,
    game::{
        clientbound_game_event_packet::{ClientboundGameEventPacket, EventType},
        clientbound_login_packet::ClientboundLoginPacket,
        clientbound_player_abilities_packet::{
            ClientboundPlayerAbilitiesPacket, PlayerAbilitiesFlags,
        },
        clientbound_player_info_update_packet::{
            ActionEnumSet, ClientboundPlayerInfoUpdatePacket, PlayerInfoEntry,
        },
        clientbound_player_position_packet::{ClientboundPlayerPositionPacket, RelativeMovements},
        clientbound_set_default_spawn_position_packet::ClientboundSetDefaultSpawnPositionPacket,
        clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket,
    },
};
use tracing::*;

use crate::{
    config::{self, ty::Location},
    network::{
        self,
        server::{constants, AServer, PlayerRef},
        GameConnection,
    },
    player::skin::SkinLayersExt,
};

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn signal_game_start(
    conn: &mut GameConnection,
    _server: &AServer, // todo
    player: &PlayerRef,
) -> network::Result<()> {
    trace!("Signaling game start to client");
    let config = config::get();
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
                game_type: *config.default_gamemode,
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

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn signal_player_update(
    conn: &mut GameConnection,
    player: &PlayerRef,
) -> network::Result<()> {
    trace!("Signaling player update to client");
    let config = config::get();
    let profile = player.lock().await.game_profile();
    conn.write(
        ClientboundPlayerInfoUpdatePacket {
            actions: ActionEnumSet {
                add_player: true,
                initialize_chat: false,
                update_game_mode: true,
                update_listed: true,
                update_latency: true,
                update_display_name: true,
            },
            entries: vec![PlayerInfoEntry {
                profile,
                listed: true,
                latency: 0,
                game_mode: *config.default_gamemode,
                display_name: None,
                chat_session: None,
            }],
        }
        .get(),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn signal_player_abilities(conn: &mut GameConnection) -> network::Result<()> {
    trace!("Signaling player update to client");
    let config = config::get();
    let creative_mode = *config.default_gamemode == GameMode::Creative;
    conn.write(
        ClientboundPlayerAbilitiesPacket {
            flags: PlayerAbilitiesFlags {
                invulnerable: false,
                flying: false,
                can_fly: config.allow_flight,
                instant_break: creative_mode,
            },
            flying_speed: 0.05,
            walking_speed: 0.1,
        }
        .get(),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(level = "trace", skip(conn), err)]
pub async fn signal_game_state_change(
    conn: &mut GameConnection,
    event: EventType,
    param: Option<f32>,
) -> network::Result<()> {
    trace!("Signaling game state change to client");
    conn.write(
        ClientboundGameEventPacket {
            event,
            param: param.unwrap_or_default(),
        }
        .get(),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn signal_spawn_position(conn: &mut GameConnection) -> network::Result<()> {
    trace!("Signaling spawn position to client");
    let pos = config::get().spawn_location;
    conn.write(
        ClientboundSetDefaultSpawnPositionPacket {
            pos: pos.to_block_pos(),
            angle: pos.yaw(),
        }
        .get(),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(level = "trace", skip(conn), fields(%location), err)]
pub async fn teleport_player(conn: &mut GameConnection, location: Location) -> network::Result<()> {
    trace!("Teleporting player to location");
    conn.write(
        ClientboundPlayerPositionPacket {
            x: location.x(),
            y: location.y(),
            z: location.z(),
            y_rot: location.yaw(),
            x_rot: location.pitch(),
            relative_arguments: RelativeMovements {
                x: false,
                y: false,
                z: false,
                y_rot: false,
                x_rot: false,
            },
            id: 1,
        }
        .get(),
    )
    .await?;
    Ok(())
}

#[tracing::instrument(level = "trace", skip_all, err)]
pub async fn signal_player_skin_layers(
    conn: &mut GameConnection,
    player: &PlayerRef,
) -> network::Result<()> {
    trace!("Signaling player skin layers to client");
    let player = player.lock().await;
    if let Some(skin) = player.skin() {
        conn.write(
            ClientboundSetEntityDataPacket {
                id: player.entity_id(),
                packed_items: EntityMetadataItems::new(vec![EntityDataItem {
                    index: 17, // https://wiki.vg/Entity_metadata#Player
                    value: EntityDataValue::Byte(skin.layers.to_bits()),
                }]),
            }
            .get(),
        )
        .await?;
    } else {
        trace!("Player has no skin to send");
    }
    Ok(())
}
