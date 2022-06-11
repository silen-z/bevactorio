use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::BuildingType;
use crate::map::{MapLayer, BuildingTileType, ACTIVE_MAP};
use crate::{BuildEvent, LastPlacedTile};

pub fn build_belt(
    mut commands: Commands,
    mut map: MapQuery,
    mut tiles: Query<&mut Tile>,
    mut events: EventReader<BuildEvent>,
    mut last_placed: ResMut<LastPlacedTile>,
) {
    for event in events
        .iter()
        .filter(|e| matches!(e.building_type, BuildingType::Belt))
    {
        if last_placed
            .map(|(_, pos)| pos == event.tile_pos)
            .unwrap_or(false)
        {
            return;
        }

        let TilePos(curr_x, curr_y) = event.tile_pos;

        let belt_dir = last_placed.map_or(BuildingTileType::BeltDown, |(_, last_pos)| match last_pos {
            TilePos(x, y) if x == curr_x - 1 && y == curr_y => BuildingTileType::BeltRight,
            TilePos(x, y) if x == curr_x + 1 && y == curr_y => BuildingTileType::BeltLeft,
            TilePos(x, y) if x == curr_x && y == curr_y - 1 => BuildingTileType::BeltUp,
            TilePos(x, y) if x == curr_x && y == curr_y + 1 => BuildingTileType::BeltDown,
            _ => BuildingTileType::BeltDown,
        });

        if let Ok(placed_entity) = map.set_tile(
            &mut commands,
            event.tile_pos,
            Tile {
                texture_index: belt_dir as u16,
                ..default()
            },
            ACTIVE_MAP,
            MapLayer::Buildings,
        ) {
            if let Some((last_e, _)) = last_placed.0 {
                if let Some(mut last_tile) = tiles
                    .get_mut(last_e)
                    .ok()
                    .filter(|t| BuildingTileType::try_from(**t).unwrap().is_belt())
                {
                    last_tile.texture_index = belt_dir as u16;
                }
            }

            last_placed.0 = Some((placed_entity, event.tile_pos));
            map.notify_chunk_for_tile(event.tile_pos, ACTIVE_MAP, MapLayer::Buildings);
        }
    }
}
