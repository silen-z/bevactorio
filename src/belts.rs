use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::BuildingType;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::BuildRequestedEvent;

#[derive(Component)]
pub struct Belt {
    pub items: [Option<(Entity, f32)>; 3],
}

const ITEM_SIZE: f32 = 1. / 3.;

impl Belt {
    pub fn place(&mut self, pos: f32, entity: Entity) -> bool {
        self.place_new(pos, || entity)
    }

    // TODO smarter placing of items on belts
    pub fn place_new(&mut self, pos: f32, entity_init: impl FnOnce() -> Entity) -> bool {
        match self.items {
            [None, None, None] => {
                self.items[0] = Some((entity_init(), pos));
            }
            [Some((_, p)), None, None] if p > pos => {
                self.items[1] = Some((entity_init(), pos));
            }
            [_, Some((_, p)), None] if p > pos => {
                self.items[2] = Some((entity_init(), pos));
            }
            _ => return false,
        }
        true
    }

    pub fn pop_first(&mut self) -> Entity {
        let popped = self.items[0].unwrap().0;

        self.items.rotate_left(1);
        self.items[2] = None;

        popped
    }
}

#[derive(Component)]
pub struct Item {
    pub belt: Entity,
    pub item_type: ItemType,
}

pub fn build_belt(
    mut commands: Commands,
    mut map_query: MapQuery,
    mut tiles: Query<&mut Tile>,
    mut events: EventReader<BuildRequestedEvent>,
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
            Ok(tile) => {
                let is_belt = tiles
                    .get(tile)
                    .map_or(false, |t| BuildingTileType::from(t.texture_index).is_belt());

                if !is_belt {
                    continue;
                }
            }
            Err(MapTileError::OutOfBounds(_)) => continue,
            _ => {}
        }

        let (belt_dir, update_last_belt) = last_placed
            .and_then(|(_, pos)| belt_dir_between(event.tile_pos, pos))
            .map(|dir| (dir, true))
            .unwrap_or((BeltDown, false));

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
                if let Some(mut last_tile) = tiles.get_mut(last_e).ok().filter(|t| {
                    update_last_belt && BuildingTileType::from(t.texture_index).is_belt()
                }) {
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

fn belt_dir_between(
    TilePos(x1, y1): TilePos,
    TilePos(x2, y2): TilePos,
) -> Option<BuildingTileType> {
    if x1 > 0 && (x2 == x1 - 1) && (y2 == y1) {
        return Some(BuildingTileType::BeltRight);
    }

    if (x2 == x1 + 1) && (y2 == y1) {
        return Some(BuildingTileType::BeltLeft);
    }

    if y1 > 0 && (x2 == x1) && y2 == (y1 - 1) {
        return Some(BuildingTileType::BeltUp);
    }

    if (x2 == x1) && (y2 == y1 + 1) {
        return Some(BuildingTileType::BeltDown);
    }

    None
}

pub fn move_items_on_belts(
    mut items: Query<&mut Transform, With<Item>>,
    belt_tiles: Query<(Entity, &TilePos, &Tile), With<Belt>>,
    mut belts: Query<(&mut Belt, &Tile)>,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
    time: Res<Time>,
) {
    for (belt_entity, belt_pos, belt_tile) in belt_tiles.iter() {
        let building_type = BuildingTileType::from(belt_tile.texture_index);

        let belt_output_pos = building_type.next_belt_pos(*belt_pos);

        if let Some(next_pos) = belt_output_pos {
            try_move_item_between_belts(
                belt_entity,
                next_pos,
                &mut map_query,
                &mut belts,
                &mut items,
                &active_map,
                time.delta_seconds(),
            );
        }

        let Ok((mut belt, _)) = belts.get_mut(belt_entity) else {
            continue;
        };

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
                let world_pos = active_map.to_world_pos(*belt_pos);
                let offset = building_type.progress_offset(*item_progress);

                transform.translation = (world_pos + offset).extend(10.);
            }
        }
    }
}

fn try_move_item_between_belts(
    belt_entity: Entity,
    next_belt_pos: TilePos,
    map_query: &mut MapQuery,
    belts: &mut Query<(&mut Belt, &Tile)>,
    items: &mut Query<&mut Transform, With<Item>>,
    active_map: &ActiveMap,
    delta: f32,
) {
    let Ok(next_belt_entity) = map_query.get_tile_entity(next_belt_pos, active_map.map_id, MapLayer::Buildings) else {
        return;
    };

    let Ok([mut belt, mut next_belt]) = belts.get_many_mut([belt_entity, next_belt_entity]) else {
        return;
    };

    let Some((first_item_entity, first_item_progress)) = belt.0.items[0] else {
        return;
    };

    let progress = first_item_progress + delta;

    if progress > 1. {
        let next_belt_type = BuildingTileType::from(next_belt.1.texture_index);

        let Some(next_belt_start) = BuildingTileType::from(belt.1.texture_index).next_belt_start(next_belt_type) else {
            return;
        };

        let next_belt_progress = (progress - 1.0 + next_belt_start).clamp(0., 1.);

        if !next_belt.0.place(next_belt_progress, first_item_entity) {
            return;
        }

        if let Ok(mut transform) = items.get_mut(first_item_entity) {
            let world_pos = active_map.to_world_pos(next_belt_pos);
            let offset = next_belt_type.progress_offset(next_belt_progress);

            transform.translation = (world_pos + offset).extend(10.);
        }

        belt.0.items.rotate_left(1);
        belt.0.items[2] = None;
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ItemType {
    Coal,
}

const MAX_INVENTORY_SIZE: usize = 8 * 8;
const STACK_SIZE: usize = 5;

#[derive(Component)]
pub struct Inventory {
    pub slots: ArrayVec<Option<(ItemType, usize)>, MAX_INVENTORY_SIZE>,
}

impl Inventory {
    pub fn insert(&mut self, amount: usize, item_type: ItemType) -> bool {
        let Some(slot) = self
            .slots
            .iter_mut()
            .find(|s| s.map_or(true, |s| s.0 == item_type && s.1 + amount <= STACK_SIZE)) else {
            return false;
        };

        *slot = match slot {
            Some((_, stored)) => Some((item_type, *stored + amount)),
            None => Some((item_type, amount)),
        };

        true
    }
}

#[derive(Component)]
pub struct BeltInput {
    pub inventory: Entity,
}

pub fn input_from_belts(
    mut commands: Commands,
    mut belts: Query<(&mut Belt, &TilePos, &Tile)>,
    items: Query<&Item>,
    mut inventories: Query<&mut Inventory>,
    inputs: Query<&BeltInput>,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
) {
    for (mut belt, belt_pos, belt_tile) in belts.iter_mut() {
        if belt.items[0].map_or(false, |(_, progress)| progress == 1.) {
            let tile_type = BuildingTileType::from(belt_tile.texture_index);

            let Some(next_pos) = tile_type.next_belt_pos(*belt_pos) else {
                continue;
            };

            let Ok(entity) = map_query.get_tile_entity(next_pos, active_map.map_id, MapLayer::Buildings) else {
                continue;
            };

            let Ok(input) = inputs.get(entity) else {
                continue;
            };

            let Ok(mut inventory) = inventories.get_mut(input.inventory) else {
                continue;
            };

            let Ok (item) = items.get(belt.items[0].unwrap().0) else {
                continue;
            };

            if inventory.insert(1, item.item_type) {
                let entity = belt.pop_first();
                commands.entity(entity).despawn();
            }
        }
    }
}
