use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::{BuildingType, BuildRequestedEvent};
use crate::map::{BuildingLayer, BuildingTileType};

const BELT_CAPACITY: usize = 3;

#[derive(Component)]
pub struct Belt {
    pub items: ArrayVec<(Entity, f32), BELT_CAPACITY>,
}

const ITEM_SIZE: f32 = 1. / BELT_CAPACITY as f32;

impl Belt {
    pub fn place(&mut self, pos: f32, entity: Entity) -> bool {
        self.place_new(pos, || entity)
    }

    // TODO smarter placing of items on belts
    pub fn place_new(&mut self, pos: f32, entity_init: impl FnOnce() -> Entity) -> bool {
        if !self.items.is_full() && self.items.last().map_or(true, |(_, p)| *p > pos) {
            self.items.push((entity_init(), pos));
            true
        } else {
            false
        }
    }
}

#[derive(Component)]
pub struct Item {
    pub belt: Entity,
    pub item_type: ItemType,
}

pub fn build_belt(
    mut commands: Commands,
    mut tiles: Query<&mut TileTextureIndex>,
    mut events: EventReader<BuildRequestedEvent>,
    mut last_placed: Local<Option<(Entity, TilePos)>>,
    mut buildings_layer_query: Query<&mut TileStorage, With<BuildingLayer>>,
) {
    use BuildingTileType::*;

    let mut building_layer = buildings_layer_query.single_mut();

    for event in events
        .iter()
        .filter(|e| matches!(e.building_type, BuildingType::Belt))
    {
        if last_placed.map_or(false, |(_, pos)| {
            pos.x == event.tile_pos.x && pos.y == event.tile_pos.y
        }) {
            continue;
        }

        match building_layer.get(&event.tile_pos) {
            Some(tile) => {
                let is_belt = tiles
                    .get(tile)
                    .map_or(false, |t| BuildingTileType::from(*t).is_belt());

                if !is_belt {
                    continue;
                }
            }
            None => {}
        }

        let (belt_dir, update_last_belt) = last_placed
            .and_then(|(_, pos)| belt_dir_between(event.tile_pos, pos))
            .map(|dir| (dir, true))
            .unwrap_or((BeltDown, false));

        let placed_belt = commands
            .spawn(Belt {
                items: ArrayVec::new(),
            })
            .id();

        building_layer.set(&event.tile_pos, placed_belt);

        if let Some((last_e, _)) = *last_placed {
            if let Some(mut last_tile) = tiles
                .get_mut(last_e)
                .ok()
                .filter(|t| update_last_belt && BuildingTileType::from(*t.as_ref()).is_belt())
            {
                last_tile.0 = belt_dir as u32;
            }
        }

        *last_placed = Some((placed_belt, event.tile_pos));
    }
}

fn belt_dir_between(
    TilePos { x: x1, y: y1 }: TilePos,
    TilePos { x: x2, y: y2 }: TilePos,
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
    belt_tiles: Query<(Entity, &TilePos, &TileTextureIndex), With<Belt>>,
    mut belts: Query<(&mut Belt, &TileTextureIndex)>,
    mut building_layer_query: Query<
        (&mut TileStorage, &TilemapTileSize, &Transform),
        (With<BuildingLayer>, Without<Item>),
    >,
    time: Res<Time>,
) {
    let (mut building_layer, tile_size, building_layer_transform) =
        building_layer_query.single_mut();

    for (belt_entity, belt_pos, belt_tile) in belt_tiles.iter() {
        let building_type = BuildingTileType::from(*belt_tile);

        let belt_output_pos = building_type.next_belt_pos(*belt_pos);

        if let Some(next_pos) = belt_output_pos {
            try_move_item_between_belts(
                belt_entity,
                next_pos,
                &mut building_layer,
                &mut belts,
                &mut items,
                time.delta_seconds(),
                tile_size,
                building_layer_transform,
            );
        }

        let Ok((mut belt, _)) = belts.get_mut(belt_entity) else {
            continue;
        };

        let mut max_progress = 1.0f32;

        for (item_entity, item_progress) in belt.items.iter_mut() {
            let next_progress = f32::clamp(
                *item_progress + time.delta_seconds(),
                0.,
                max_progress.max(0.),
            );
            *item_progress = next_progress;
            max_progress = next_progress - ITEM_SIZE;

            if let Ok(mut transform) = items.get_mut(*item_entity) {
                let world_pos = tile_to_world_pos(*belt_pos, tile_size, building_layer_transform);
                let offset = building_type.progress_offset(*item_progress);

                transform.translation = (world_pos + offset).extend(10.);
            }
        }
    }
}

fn try_move_item_between_belts(
    belt_entity: Entity,
    next_belt_pos: TilePos,
    building_layer: &TileStorage,
    belts: &mut Query<(&mut Belt, &TileTextureIndex)>,
    items: &mut Query<&mut Transform, With<Item>>,
    delta: f32,
    tile_size: &TilemapTileSize,
    building_layer_transform: &Transform,
) {
    let Some(next_belt_entity) = building_layer.get(&next_belt_pos) else {
        return;
    };

    let Ok([(mut belt, belt_tile), (mut next_belt, next_belt_tile)]) = belts.get_many_mut([belt_entity, next_belt_entity]) else {
        return;
    };

    let Some(&(first_item_entity, first_item_progress)) = belt.items.first() else {
        return;
    };

    let progress = first_item_progress + delta;

    if progress > 1. {
        let next_belt_type = BuildingTileType::from(*next_belt_tile);

        let Some(next_belt_start) = BuildingTileType::from(*belt_tile).next_belt_start(next_belt_type) else {
            return;
        };

        let next_belt_progress = (progress - 1.0 + next_belt_start).clamp(0., 1.);

        if !next_belt.place(next_belt_progress, first_item_entity) {
            return;
        }

        let (first_item_entity, _) = belt.items.pop_at(0).unwrap();

        if let Ok(mut transform) = items.get_mut(first_item_entity) {
            let world_pos = tile_to_world_pos(next_belt_pos, tile_size, building_layer_transform);
            let offset = next_belt_type.progress_offset(next_belt_progress);

            transform.translation = (world_pos + offset).extend(10.);
        }
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
    mut belts: Query<(&mut Belt, &TilePos, &TileTextureIndex)>,
    items: Query<&Item>,
    mut inventories: Query<&mut Inventory>,
    inputs: Query<&BeltInput>,
    building_layer_query: Query<&TileStorage, With<BuildingLayer>>,
) {
    let building_layer = building_layer_query.single();

    for (mut belt, belt_pos, belt_tile) in belts.iter_mut() {
        let Some((item_entity, progress)) = belt.items.first() else {
            continue;
        };

        if *progress == 1. {
            let tile_type = BuildingTileType::from(*belt_tile);

            let Some(next_pos) = tile_type.next_belt_pos(*belt_pos) else {
                continue;
            };

            let Some(entity) = building_layer.get(&next_pos) else {
                continue;
            };

            let Ok(input) = inputs.get(entity) else {
                continue;
            };

            let Ok(mut inventory) = inventories.get_mut(input.inventory) else {
                continue;
            };

            let Ok (item) = items.get(*item_entity) else {
                continue;
            };

            if inventory.insert(1, item.item_type) {
                let (entity, _) = belt.items.pop_at(0).unwrap();
                commands.entity(entity).despawn();
            }
        }
    }
}

fn tile_to_world_pos(
    tile_pos: TilePos,
    tile_size: &TilemapTileSize,
    tilemap_transform: &Transform,
) -> Vec2 {
    let x = tile_pos.x as f32 * tile_size.x + tilemap_transform.translation.x;
    let y = tile_pos.y as f32 * tile_size.y + tilemap_transform.translation.y;

    Vec2::new(x, y)
}
