use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::belts::{Belt, Item};
use crate::buildings::{Building, BuildingTile, BuildingType};
use crate::map::{ActiveMap, MapLayer};
use crate::BuildEvent;

#[derive(Component)]
pub struct Mine {
    timer: Timer,
    output: TilePos,
}

pub fn build_mine(
    mut commands: Commands,
    mut map_query: MapQuery,
    mut events: EventReader<BuildEvent>,
    active_map: Res<ActiveMap>,
) {
    for event in events
        .iter()
        .filter(|e| matches!(e.building_type, BuildingType::Mine))
    {
        let building_template = event.building_type.template(event.tile_pos);

        let possible_to_build = building_template.positions().all(|tile_pos| {
            let res = map_query.get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings);
            matches!(res, Err(MapTileError::NonExistent(_)))
        });

        if !possible_to_build {
            continue;
        }

        let mine_entity = commands
            .spawn()
            .insert(Mine {
                output: event.tile_pos,
                timer: Timer::from_seconds(1.5, true),
            })
            .id();

        let mut tiles = ArrayVec::new();

        for (tile_type, tile_pos) in building_template.instructions() {
            if let Ok(mine_tile_entity) = map_query.set_tile(
                &mut commands,
                tile_pos,
                Tile {
                    texture_index: tile_type as u16,
                    ..default()
                },
                active_map.map_id,
                MapLayer::Buildings,
            ) {
                commands.entity(mine_tile_entity).insert(BuildingTile {
                    building_entity: mine_entity,
                });
                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::Buildings);
                tiles.push((tile_pos, mine_tile_entity));
            }
        }

        commands.entity(mine_entity).insert(Building { tiles });
    }
}

pub fn mine_produce(
    mut commands: Commands,
    mut mines: Query<&mut Mine>,
    mut belts: Query<(Entity, &mut Belt)>,
    mut map_query: MapQuery,
    time: Res<Time>,
    asset_server: ResMut<AssetServer>,
    active_map: Res<ActiveMap>,
) {
    for mut mine in mines.iter_mut() {
        if mine.timer.tick(time.delta()).just_finished() {
            for e in map_query
                .get_tile_neighbors(mine.output, active_map.map_id, MapLayer::Buildings)
                .into_iter()
                .flatten()
            {
                if let Ok((belt_entity, mut belt)) = belts.get_mut(e) {
                    if let Some(slot) = belt.space(0.33) {
                        let item_entity = commands
                            .spawn_bundle(SpriteBundle {
                                transform: Transform::from_xyz(0., 0., -9999.),
                                texture: asset_server.load("items.png"),
                                ..default()
                            })
                            .insert(Item {
                                belt: belt_entity,
                                progress: 0.,
                            })
                            .id();

                        *slot = Some((item_entity, 0.));
                        break;
                    }
                }
            }
        }
    }
}
