use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::map::{MapLayer, ACTIVE_MAP};

#[derive(Copy, Clone, Debug)]
pub enum BuildingType {
    Belt,
    Mine,
}

pub const AVAILABLE_BUILDINGS: &[BuildingType] = &[BuildingType::Belt, BuildingType::Mine];

#[derive(Default)]
pub struct SelectedBuilding(usize);

impl SelectedBuilding {
    pub fn prev(&mut self) {
        if self.0 == 0 {
            self.0 = AVAILABLE_BUILDINGS.len() - 1;
        } else {
            self.0 -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.0 == AVAILABLE_BUILDINGS.len() - 1 {
            self.0 = 0;
        } else {
            self.0 += 1;
        }
    }

    pub fn get(&self) -> BuildingType {
        AVAILABLE_BUILDINGS[self.0]
    }
}

pub struct BuildEvent {
    pub building_type: BuildingType,
    pub tile_pos: TilePos,
}

pub struct DemolishEvent {
    pub tile_pos: TilePos,
}


pub fn demolish_building(
    mut commands: Commands,
    mut map: MapQuery,
    mut events: EventReader<DemolishEvent>,
) {
    for event in events.iter() {
        if let Ok(_) = map.despawn_tile(
            &mut commands,
            event.tile_pos,
            ACTIVE_MAP,
            MapLayer::Buildings,
        ) {
            map.notify_chunk_for_tile(event.tile_pos, ACTIVE_MAP, MapLayer::Buildings);
        }
    }
}
