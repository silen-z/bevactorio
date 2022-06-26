use bevy::prelude::*;

use super::{BuildingBuiltEvent, BuildingType};
use crate::belts::{BeltInput, Inventory};

pub fn build_chest(mut commands: Commands, mut new_buildings: EventReader<BuildingBuiltEvent>) {
    for event in new_buildings.iter() {
        if let BuildingType::Chest = event.building_type {
            let (tile_entity, _, _) = event.layout.tiles[0];

            commands.entity(tile_entity).insert(BeltInput {
                inventory: event.entity,
            });

            commands.entity(event.entity).insert(Inventory {
                slots: (&[None; 1] as &[_]).try_into().unwrap(),
            });
        }
    }
}
