use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::belts::{BeltInput, Inventory};
use crate::buildings::{Building, BuildingType};
use crate::map::BuildingLayer;

pub fn build_chest(
    mut commands: Commands,
    new_chests: Query<(Entity, &BuildingType, &TilePos), Added<Building>>,
    building_layer: Query<&TileStorage, With<BuildingLayer>>,
) {
    let mut building_layer = building_layer.single();

    for (entity, building_type, tile_pos) in new_chests.iter() {
        if let BuildingType::Chest = building_type {
            if let Some(tile_entity) = building_layer.get(tile_pos) {
                commands
                    .entity(tile_entity)
                    .insert(BeltInput { inventory: entity })
                    .insert(Inventory {
                        slots: (&[None; 1] as &[_]).try_into().unwrap(),
                    });
            }
        }
    }
}
