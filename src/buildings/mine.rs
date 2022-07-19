use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::belts::{Belt, Item, ItemType};
use crate::buildings::BuildingType;
use crate::map::{ActiveMap, MapLayer};

use super::Building;

#[derive(Component)]
pub struct Mine {
    timer: Timer,
    output: TilePos,
}

pub fn build_mine(
    mut commands: Commands,
    new_buildings: Query<(Entity, &BuildingType, &TilePos), Added<Building>>,
) {
    for (entity, building_type, tile_pos) in new_buildings.iter() {
        if let BuildingType::Mine = building_type {
            info!("built mine");
            commands.entity(entity).insert(Mine {
                timer: Timer::new(Duration::from_secs(1), true),
                output: *tile_pos,
            });
        }
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
            let ouputs = output_positions(mine.output).flat_map(|pos| {
                map_query.get_tile_entity(pos, active_map.map_id, MapLayer::Buildings)
            });

            for belt_entity in ouputs {
                if let Ok((belt_entity, mut belt)) = belts.get_mut(belt_entity) {
                    if belt.place_new(0.33, || {
                        commands
                            .spawn_bundle(SpriteBundle {
                                transform: Transform::from_xyz(0., 0., -9999.),
                                texture: asset_server.load("items.png"),
                                ..default()
                            })
                            .insert(Item {
                                belt: belt_entity,
                                item_type: ItemType::Coal,
                            })
                            .id()
                    }) {
                        break;
                    }
                }
            }
        }
    }
}

fn output_positions(output: TilePos) -> impl Iterator<Item = TilePos> {
    [
        (output.0 > 0).then(|| TilePos(output.0 - 1, output.1)),
        (output.1 > 0).then(|| TilePos(output.0, output.1 - 1)),
    ]
    .into_iter()
    .flatten()
}
