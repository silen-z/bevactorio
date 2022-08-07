use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::{BuildingBuiltEvent, BuildingType};
use crate::belts::{Belt, Item, ItemType};
use crate::map::{BuildingLayer, BuildingTileType};

#[derive(Component)]
pub struct Mine {
    timer: Timer,
    output: TilePos,
}

pub fn build_mine(mut commands: Commands, mut new_buildings: EventReader<BuildingBuiltEvent>) {
    for event in new_buildings.iter() {
        if let BuildingType::Mine = event.building_type {
            let (_, output, _) = event
                .layout
                .tiles
                .iter()
                .find(|(_, _, tile_type)| matches!(tile_type, BuildingTileType::MineBottomLeft))
                .unwrap();

            commands.entity(event.entity).insert(Mine {
                timer: Timer::new(Duration::from_secs(1), true),
                output: *output,
            });
        }
    }
}

pub fn mine_produce(
    mut commands: Commands,
    mut mines: Query<&mut Mine>,
    mut belts: Query<(Entity, &mut Belt)>,
    mut tilemap_query: Query<&TileStorage, With<BuildingLayer>>,
    time: Res<Time>,
    asset_server: ResMut<AssetServer>,
) {
    let building_layer = tilemap_query.single();

    for mut mine in mines.iter_mut() {
        if mine.timer.tick(time.delta()).just_finished() {
            let ouputs = output_positions(mine.output).flat_map(|pos| building_layer.get(&pos));

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
        (output.x > 0).then(|| TilePos::new(output.x - 1, output.y)),
        (output.y > 0).then(|| TilePos::new(output.x, output.y - 1)),
    ]
    .into_iter()
    .flatten()
}
