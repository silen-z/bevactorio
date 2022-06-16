use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::BuildingType;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::BuildEvent;

#[derive(Component)]
pub struct Belt {
    pub items: [Option<(Entity, f32)>; 3],
}

const ITEM_SIZE: f32 = 1. / 3.;

impl Belt {
    // TODO better select space here, and probably take closure instead 
    pub fn space(&mut self, pos: f32) -> Option<&mut Option<(Entity, f32)>> {
        match self.items {
            [None, None, None] => self.items.get_mut(0),
            [Some((_, p)), None, None] if p > pos => self.items.get_mut(1),
            [_, Some((_, p)), None] if p > pos => self.items.get_mut(2),
            _ => None,
        }
    }
}

#[derive(Component)]
pub struct Item {
    pub belt: Entity,
    pub progress: f32,
}

pub fn build_belt(
    mut commands: Commands,
    mut map_query: MapQuery,
    mut tiles: Query<&mut Tile>,
    mut events: EventReader<BuildEvent>,
    mut last_placed: Local<Option<(Entity, TilePos)>>,
    active_map: Res<ActiveMap>,
) {
    use BuildingTileType::*;

    for event in events
        .iter()
        .filter(|e| matches!(e.building_type, BuildingType::Belt))
    {
        if last_placed.map_or(false, |(_, pos)| pos == event.tile_pos) {
            continue;
        }

        match map_query.get_tile_entity(event.tile_pos, active_map.map_id, MapLayer::Buildings) {
            Ok(tile) if !BuildingTileType::from(*tiles.get(tile).unwrap()).is_belt() => continue,
            Err(MapTileError::OutOfBounds(_)) => continue,
            Err(MapTileError::AlreadyExists(_)) => unreachable!(),
            _ => {}
        }

        let TilePos(curr_x, curr_y) = event.tile_pos;

        let (belt_dir, update_last_belt) = match *last_placed {
            Some((_, TilePos(x, y))) if curr_x > 0 && (x == curr_x - 1) && (y == curr_y) => {
                (BeltRight, true)
            }
            Some((_, TilePos(x, y))) if (x == curr_x + 1) && (y == curr_y) => (BeltLeft, true),
            Some((_, TilePos(x, y))) if curr_y > 0 && (x == curr_x) && y == (curr_y - 1) => {
                (BeltUp, true)
            }
            Some((_, TilePos(x, y))) if (x == curr_x) && (y == curr_y + 1) => (BeltDown, true),
            _ => (BeltDown, false),
        };

        if let Ok(placed_entity) = map_query.set_tile(
            &mut commands,
            event.tile_pos,
            Tile {
                texture_index: belt_dir as u16,
                ..default()
            },
            active_map.map_id,
            MapLayer::Buildings,
        ) {
            if let Some((last_e, last_pos)) = *last_placed {
                if let Some(mut last_tile) = tiles
                    .get_mut(last_e)
                    .ok()
                    .filter(|t| update_last_belt && BuildingTileType::from(**t).is_belt())
                {
                    last_tile.texture_index = belt_dir as u16;
                    map_query.notify_chunk_for_tile(
                        last_pos,
                        active_map.map_id,
                        MapLayer::Buildings,
                    );
                }
            }

            commands
                .entity(placed_entity)
                .insert(Belt { items: [None; 3] });

            *last_placed = Some((placed_entity, event.tile_pos));
            map_query.notify_chunk_for_tile(event.tile_pos, active_map.map_id, MapLayer::Buildings);
        }
    }
}

pub fn move_items_on_belts(
    // mut commands: Commands,
    mut items: Query<&mut Transform, With<Item>>,
    belt_tiles: Query<(Entity, &TilePos, &Tile), With<Belt>>,
    mut belts: Query<(&mut Belt, &Tile)>,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
    time: Res<Time>,
) {
    for (belt_entity, belt_pos, belt_tile) in belt_tiles.iter() {
        let (mut belt, next_belt) =
            match next_belt(belt_tile, *belt_pos, &mut map_query, &active_map) {
                Some((next_belt_pos, next_belt_entity)) => {
                    match belts.get_many_mut([belt_entity, next_belt_entity]) {
                        Ok([belt, next_belt]) => {
                            (belt.0, Some((next_belt.0, next_belt.1, next_belt_pos)))
                        }
                        _ => continue,
                    }
                }
                _ => {
                    let belt = belts.get_mut(belt_entity).unwrap();
                    (belt.0, None)
                }
            };

        if let Some((mut next_belt, next_belt_tile, next_belt_pos)) = next_belt {
            move_first_item(
                &mut belt,
                belt_tile,
                &mut next_belt,
                next_belt_tile,
                next_belt_pos,
                &active_map,
                &mut items,
                time.delta_seconds(),
            );
        }

        let mut max_progress = 1.0f32;

        for (item_entity, item_progress) in belt.items.iter_mut().flatten() {
            let next_progress = f32::clamp(
                *item_progress + time.delta_seconds(),
                0.,
                max_progress.max(0.),
            );
            *item_progress = next_progress;
            max_progress = next_progress - ITEM_SIZE;

            if let Ok(mut transform) = items.get_mut(*item_entity) {
                transform.translation =
                    calculate_world_pos(&active_map, belt_tile, *belt_pos, *item_progress);
            }
        }
    }
}

fn calculate_world_pos(
    active_map: &ActiveMap,
    tile: &Tile,
    tile_pos: TilePos,
    progress: f32,
) -> Vec3 {
    use BuildingTileType::*;

    fn lerp(n1: f32, n2: f32, scalar: f32) -> f32 {
        n1 + (n2 - n1) * scalar
    }

    let world_pos = active_map.to_world_pos(tile_pos);

    let progress_offset = match BuildingTileType::from(tile.texture_index) {
        BeltUp => Vec2::new(8., lerp(0., 16., progress)),
        BeltDown => Vec2::new(8., lerp(16., 0., progress)),
        BeltLeft => Vec2::new(lerp(16., 0., progress), 8.),
        BeltRight => Vec2::new(lerp(0., 16., progress), 8.),
        _ => panic!("not a belt"),
    };

    (world_pos + progress_offset).extend(10.)
}

fn next_belt(
    belt_tile: &Tile,
    TilePos(belt_x, belt_y): TilePos,
    map_query: &mut MapQuery,
    active_map: &ActiveMap,
) -> Option<(TilePos, Entity)> {
    use BuildingTileType::*;
    let next_belt_pos = match BuildingTileType::from(belt_tile.texture_index) {
        BeltUp => TilePos(belt_x, belt_y + 1),
        BeltDown if belt_y > 0 => TilePos(belt_x, belt_y - 1),
        BeltLeft if belt_x > 0 => TilePos(belt_x - 1, belt_y),
        BeltRight => TilePos(belt_x + 1, belt_y),
        _ => return None,
    };

    let next_belt_entity = map_query
        .get_tile_entity(next_belt_pos, active_map.map_id, MapLayer::Buildings)
        .ok()?;

    Some((next_belt_pos, next_belt_entity))
}

fn move_first_item(
    belt: &mut Belt,
    belt_tile: &Tile,

    next_belt: &mut Belt,
    next_belt_tile: &Tile,
    next_belt_pos: TilePos,

    active_map: &ActiveMap,
    items: &mut Query<&mut Transform, With<Item>>,
    delta: f32,
) -> Option<()> {
    let (first_item_entity, first_item_progress) = belt.items[0]?;
    let progress = first_item_progress + delta;

    if progress > 1. {
        let next_belt_start = next_belt_start(belt_tile, next_belt_tile)?;
        let next_belt_progress = (progress - 1.0 + next_belt_start).clamp(0., 1.);

        let slot = next_belt.space(next_belt_progress)?;

        *slot = Some((first_item_entity, next_belt_progress));

        if let Ok(mut transform) = items.get_mut(first_item_entity) {
            transform.translation =
                calculate_world_pos(&active_map, belt_tile, next_belt_pos, next_belt_progress);
        }

        belt.items.rotate_left(1);
        belt.items[2] = None;

        Some(())
    } else {
        None
    }
}

fn next_belt_start(belt_tile: &Tile, next_belt_tile: &Tile) -> Option<f32> {
    use BuildingTileType::*;
    match (
        belt_tile.texture_index.into(),
        next_belt_tile.texture_index.into(),
    ) {
        (BeltDown | BeltUp, BeltLeft | BeltRight) => Some(0.5),
        (BeltLeft | BeltRight, BeltDown | BeltUp) => Some(0.5),
        (x, y) if x == y => Some(0.0),
        _ => None,
    }
}
