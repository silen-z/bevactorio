use core::panic;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::BuildingType;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::BuildEvent;

#[derive(Component)]
pub struct Belt {
    pub items: [Option<(Entity, f32)>; 3],
}

impl Belt {
    fn back_space(&self) -> f32 {
        let last_full_slot = self.items.into_iter().rev().find_map(|s| s);
        last_full_slot.map_or(1., |(_, progress)| progress - 0.3333333332 * 0.5)
    }

    fn push_back(&mut self, item: Entity, progress: f32) {
        let first_empty_slot = if let Some(e) = self.items.into_iter().position(|s| s.is_none()) {
            e
        } else {
            panic!("{:?} {:?} ", self.items, self.back_space());
        };
        self.items[first_empty_slot] = Some((item, progress));
    }

    pub fn space(&mut self, pos: f32) -> Option<&mut Option<(Entity, f32)>> {
        debug!("{:?}", self.items);
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
            Some((_, TilePos(x, y))) if (x == curr_x - 1) && (y == curr_y) => (BeltRight, true),
            Some((_, TilePos(x, y))) if (x == curr_x + 1) && (y == curr_y) => (BeltLeft, true),
            Some((_, TilePos(x, y))) if (x == curr_x) && y == (curr_y - 1) => (BeltUp, true),
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
    mut belts: Query<&mut Belt>,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
    time: Res<Time>,
) {
    use BuildingTileType::*;

    for (belt_entity, belt_pos, belt_tile) in belt_tiles.iter() {
        let next_belt_pos = match BuildingTileType::from(belt_tile.texture_index) {
            BeltUp => Some(TilePos(belt_pos.0, belt_pos.1 + 1)),
            BeltDown if belt_pos.1 > 0 => Some(TilePos(belt_pos.0, belt_pos.1 - 1)),
            BeltLeft if belt_pos.0 > 0 => Some(TilePos(belt_pos.0 - 1, belt_pos.1)),
            BeltRight => Some(TilePos(belt_pos.0 + 1, belt_pos.1)),
            _ => None,
        };

        let (mut belt, mut next_belt) = match next_belt_pos.and_then(|nbp| {
            map_query
                .get_tile_entity(nbp, active_map.map_id, MapLayer::Buildings)
                .ok()
        }) {
            Some(next_belt_entity) => {
                if let Ok([belt, next_belt]) = belts.get_many_mut([belt_entity, next_belt_entity]) {
                    (belt, Some(next_belt))
                } else {
                    continue;
                }
            }
            _ => {
                let belt = belts.get_mut(belt_entity).unwrap();
                (belt, None)
            }
        };

        let mut move_to_next = false;

        let mut max_progress = next_belt.as_ref().map_or(1., |nb| 1. + nb.back_space());

        for slot in belt.items.iter_mut() {
            let (item_entity, item_progress) = match slot {
                Some((e, i)) => (*e, i),
                None => {
                    break;
                }
            };

            let next_progress = f32::min(max_progress, *item_progress + time.delta_seconds());

            let (tile_pos, progress) = match next_belt.take() {
                Some(mut nb) if next_progress > 1. && nb.items.iter().any(|s| s.is_none()) => {
                    *slot = None;
                    let next_belt_progress = next_progress - 1.;
                    nb.push_back(item_entity, next_belt_progress);
                    move_to_next = true;
                    max_progress = next_belt_progress;
                    (next_belt_pos.unwrap(), next_belt_progress)
                }
                _ => {
                    let next_progress = f32::min(1.0, next_progress);
                    *item_progress = next_progress;
                    max_progress = next_progress - 0.333333334;
                    (*belt_pos, next_progress)
                }
            };

            if let Ok(mut transform) = items.get_mut(item_entity) {
                let world_pos = active_map.to_world_pos(tile_pos);

                let progress_offset = match BuildingTileType::from(*belt_tile) {
                    BeltUp => Vec2::new(8., lerp(0., 16., progress)),
                    BeltDown => Vec2::new(8., lerp(16., 0., progress)),
                    BeltLeft => Vec2::new(lerp(16., 0., progress), 8.),
                    BeltRight => Vec2::new(lerp(0., 16., progress), 8.),
                    _ => continue,
                };

                transform.translation = (world_pos + progress_offset).extend(10.);
                // } else {
                //     info!("item not yet spawn?");
            }
        }

        if move_to_next {
            belt.items.rotate_left(1);
            belt.items[2] = None;
        }
    }
}

fn lerp(n1: f32, n2: f32, scalar: f32) -> f32 {
    n1 + (n2 - n1) * scalar
}
