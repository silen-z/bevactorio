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
    // mut commands: Commands,
    mut items: Query<&mut Transform, With<Item>>,
    belt_tiles: Query<(Entity, &TilePos, &Tile), With<Belt>>,
    mut belts: Query<(&mut Belt, &Tile)>,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
    time: Res<Time>,
) {
    for (belt_entity, belt_pos, belt_tile) in belt_tiles.iter() {
        let (mut belt, next_belt) = if let Some(pair) = belt_pair(
            belt_entity,
            belt_tile,
            *belt_pos,
            &mut belts,
            &mut map_query,
            &active_map,
        ) {
            (pair.0, Some(pair.1))
        } else if let Ok((belt, _)) = belts.get_mut(belt_entity) {
            (belt, None)
        } else {
            continue;
        };

        if let Some(mut nb) = next_belt {
            move_first_item(
                &mut belt,
                belt_tile,
                &mut nb.0,
                nb.1,
                nb.2,
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

fn belt_pair<'q>(
    belt_entity: Entity,
    belt_tile: &Tile,
    belt_pos: TilePos,
    belts: &'q mut Query<(&mut Belt, &Tile)>,
    map_query: &mut MapQuery,
    active_map: &ActiveMap,
) -> Option<(Mut<'q, Belt>, (Mut<'q, Belt>, &'q Tile, TilePos))> {
    let next_belt_pos = BuildingTileType::from(belt_tile.texture_index).next_belt_pos(belt_pos)?;
    let next_belt_entity = map_query
        .get_tile_entity(next_belt_pos, active_map.map_id, MapLayer::Buildings)
        .ok()?;

    if let Ok([belt, next_belt]) = belts.get_many_mut([belt_entity, next_belt_entity]) {
        let next_belt = (next_belt.0, next_belt.1, next_belt_pos);
        return Some((belt.0, next_belt));
    }

    None
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
) {
    let Some((first_item_entity, first_item_progress)) = belt.items[0] else {
        return;
    };

    let progress = first_item_progress + delta;

    if progress > 1. {
        let Some(next_belt_start) = BuildingTileType::from(belt_tile.texture_index).next_belt_start(next_belt_tile.texture_index) else {
            return;
        };

        let next_belt_progress = (progress - 1.0 + next_belt_start).clamp(0., 1.);

        if !next_belt.place(next_belt_progress, first_item_entity) {
            return;
        }

        if let Ok(mut transform) = items.get_mut(first_item_entity) {
            transform.translation =
                calculate_world_pos(&active_map, belt_tile, next_belt_pos, next_belt_progress);
        }

        belt.items.rotate_left(1);
        belt.items[2] = None;
    }
}
