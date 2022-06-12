use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::BuildingType;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::{BuildEvent, LastPlacedTile};

#[derive(Component)]
pub struct Belt {
    pub item: Option<Entity>,
}

#[derive(Component)]
pub struct Item {
    pub belt: Entity,
    pub progress: f32,
}

pub fn build_belt(
    mut commands: Commands,
    mut map: MapQuery,
    mut tiles: Query<&mut Tile>,
    mut events: EventReader<BuildEvent>,
    mut last_placed: ResMut<LastPlacedTile>,
    active_map: Res<ActiveMap>,
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

        let belt_dir =
            last_placed.map_or(BuildingTileType::BeltDown, |(_, last_pos)| match last_pos {
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
            active_map.map_id,
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

            commands.entity(placed_entity).insert(Belt { item: None });

            last_placed.0 = Some((placed_entity, event.tile_pos));
            map.notify_chunk_for_tile(event.tile_pos, active_map.map_id, MapLayer::Buildings);
        }
    }
}

pub fn move_items_on_belts(
    mut commands: Commands,
    mut items: Query<(Entity, &mut Item, &mut Transform)>,
    mut belts: Query<(&mut Belt, &TilePos, &Tile)>,
    mut map_query: MapQuery,
    time: Res<Time>,
    active_map: Res<ActiveMap>,
) {
    use BuildingTileType::*;

    for (item_entity, mut item, mut item_transform) in items.iter_mut() {
        let (mut belt, belt_pos, belt_tile) = match belts.get_mut(item.belt) {
            Ok(b) => b,
            _ => {
                commands.entity(item_entity).despawn();
                continue;
            }
        };

        item.progress += time.delta_seconds();

        let next_belt_pos = if item.progress > 1.0 {
            let next_belt_pos = match BuildingTileType::try_from(*belt_tile) {
                Ok(BeltUp) => TilePos(belt_pos.0, belt_pos.1 + 1),
                Ok(BeltDown) => TilePos(belt_pos.0, belt_pos.1 - 1),
                Ok(BeltLeft) => TilePos(belt_pos.0 - 1, belt_pos.1),
                Ok(BeltRight) => TilePos(belt_pos.0 + 1, belt_pos.1),
                _ => panic!("item not on belt tile"),
            };

            match map_query.get_tile_entity(next_belt_pos, active_map.map_id, MapLayer::Buildings) {
                Ok(next_belt_entity) => {
                    item.belt = next_belt_entity;
                    item.progress -= 1.0;
                    belt.item = None;
                    Some(next_belt_pos)
                }
                _ => {
                    item.progress = 1.0;
                    None
                }
            }
        } else {
            None
        };

        let world_pos = active_map.to_world_pos(next_belt_pos.unwrap_or(*belt_pos));

        let progress_transform = match BuildingTileType::try_from(*belt_tile) {
            Ok(BeltUp) => Vec3::new(8., lerp(0., 16., item.progress), 0.),
            Ok(BeltDown) => Vec3::new(8., lerp(16., 0., item.progress), 0.),
            Ok(BeltLeft) => Vec3::new(lerp(16., 0., item.progress), 8., 0.),
            Ok(BeltRight) => Vec3::new(lerp(0., 16., item.progress), 8., 0.),
            _ => continue,
        };

        item_transform.translation = world_pos.extend(10.0) + progress_transform;
    }
}

fn lerp(n1: f32, n2: f32, scalar: f32) -> f32 {
    n1 + (n2 - n1) * scalar
}
