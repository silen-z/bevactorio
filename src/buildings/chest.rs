use bevy::prelude::*;
use bevy_ecs_tilemap::*;

use crate::belts::{BeltInput, Inventory};
use crate::buildings::{Building, BuildingType};
use crate::map::{ActiveMap, MapLayer};

pub fn build_chest(
    mut commands: Commands,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
    new_chests: Query<(Entity, &BuildingType, &TilePos), Added<Building>>,
) {
    for (entity, building_type, tile_pos) in new_chests.iter() {
        if let BuildingType::Chest = building_type {
            if let Ok(tile_entity) =
                map_query.get_tile_entity(*tile_pos, active_map.map_id, MapLayer::Buildings)
            {
                commands
                    .entity(tile_entity)
                    .insert(BeltInput { inventory: entity });

                commands.entity(entity).insert(Inventory {
                    slots: (&[None; 1] as &[_]).try_into().unwrap(),
                });
            }
        }
    }
}
