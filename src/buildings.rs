use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::input::MapCursorPos;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};

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
    mut map_query: MapQuery,
    mut events: EventReader<DemolishEvent>,
    active_map: Res<ActiveMap>,
) {
    for event in events.iter() {
        if let Ok(_) = map_query.despawn_tile(
            &mut commands,
            event.tile_pos,
            active_map.map_id,
            MapLayer::Buildings,
        ) {
            map_query.notify_chunk_for_tile(event.tile_pos, active_map.map_id, MapLayer::Buildings);
        }
    }
}

#[derive(Component)]
pub struct BuildGuide;

pub fn update_build_guide(
    mut commands: Commands,
    mut map_query: MapQuery,
    mouse_pos: Res<MapCursorPos>,
    build_guides: Query<&TilePos, With<BuildGuide>>,
    active_map: Res<ActiveMap>,
    selected_building: Res<SelectedBuilding>,
) {
    if !mouse_pos.is_changed() && !selected_building.is_changed() {
        return;
    }

    for tile_pos in build_guides.iter() {
        let _ = map_query.despawn_tile(
            &mut commands,
            *tile_pos,
            active_map.map_id,
            MapLayer::BuildGuide,
        );
        map_query.notify_chunk_for_tile(*tile_pos, active_map.map_id, MapLayer::BuildGuide);
    }

    if let Some(tile_pos) = mouse_pos.0 {
        for (building_type, offset) in selected_building.get().tiles() {
            let tile = Tile {
                texture_index: building_type as u16,
                color: Color::rgba(1., 0., 0., 0.5),
                ..default()
            };

            if let Ok(entity) = map_query.set_tile(
                &mut commands,
                TilePos(tile_pos.0 + offset.0, tile_pos.1 + offset.1),
                tile,
                active_map.map_id,
                MapLayer::BuildGuide,
            ) {
                commands.entity(entity).insert(BuildGuide);
                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::BuildGuide);
            }
        }
    }
}

use BuildingTileType::*;

#[rustfmt::skip]
const BELT_TEMPLATE: [Option<BuildingTileType>; 9] = [
    Some(BeltUp), None, None,
    None,         None, None,
    None,         None, None
];

#[rustfmt::skip]
const MINE_TEMPLATE: [Option<BuildingTileType>; 9] = [
    Some(MineBottomLeft),  Some(MineBottomRight), None,
    Some(MineTopLeft),     Some(MineTopRight),    None,
    None,                  None,                  None,
];

impl BuildingType {
    pub fn tiles(&self) -> impl Iterator<Item = (BuildingTileType, TilePos)> {
        let template = match self {
            BuildingType::Belt => BELT_TEMPLATE,
            BuildingType::Mine => MINE_TEMPLATE,
        };

        template.into_iter().enumerate().flat_map(|(i, tile)| {
            tile.map(|t| {
                let tile_pos = TilePos(i as u32 % 3, i as u32 / 3);
                (t, tile_pos)
            })
        })
    }
}
