use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::belts::{Belt, Item};
use crate::buildings::BuildingType;
use crate::map::{ActiveMap, MapLayer};
use crate::BuildEvent;

#[derive(Component)]
pub struct Mine {
    timer: Timer,
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
        let building_tiles = event.building_type.tiles(event.tile_pos);

        if building_tiles.clone().any(|(_, tile_pos)| {
            let res = map_query.get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings);
            matches!(res, Ok(_) | Err(MapTileError::OutOfBounds(_)))
        }) {
            continue;
        }

        for (tile_type, tile_pos) in building_tiles {
            if let Ok(mine_entity) = map_query.set_tile(
                &mut commands,
                tile_pos,
                Tile {
                    texture_index: tile_type as u16,
                    ..default()
                },
                active_map.map_id,
                MapLayer::Buildings,
            ) {
                commands.entity(mine_entity).insert(Mine {
                    timer: Timer::from_seconds(0.5, true),
                });

                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::Buildings);
            }
        }
    }
}

pub fn mine_produce(
    mut commands: Commands,
    mut mines: Query<(&mut Mine, &TilePos)>,
    mut set: ParamSet<(Query<(Entity, &mut Belt)>, MapQuery)>,
    time: Res<Time>,
    asset_server: ResMut<AssetServer>,
    active_map: Res<ActiveMap>,
) {
    for (mut mine, mine_pos) in mines.iter_mut() {
        if mine.timer.tick(time.delta()).just_finished() {
            for e in set
                .p1()
                .get_tile_neighbors(*mine_pos, active_map.map_id, MapLayer::Buildings)
                .into_iter()
                .flatten()
            {
                if let Ok((belt_entity, mut belt)) = set.p0().get_mut(e) {
                    if belt.item.is_some() {
                        continue;
                    }

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

                    belt.item = Some(item_entity);
                    break;
                }
            }
        }
    }
}
