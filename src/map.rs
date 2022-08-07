use std::num::ParseIntError;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::Building;
use crate::camera::Zoom;

#[derive(Component)]
pub struct TerrainLayer;

#[derive(Component)]
pub struct BuildingLayer;

#[derive(Component)]
pub struct GridLayer;

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum TerrainType {
    Grass = 0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum BuildingTileType {
    BeltUp = 0,
    BeltDown = 1,
    BeltLeft = 2,
    BeltRight = 3,
    MineTopLeft = 4,
    MineTopRight = 5,
    MineBottomLeft = 6,
    MineBottomRight = 7,
    Explosion = 8,
    Chest = 9,
    Unknown = u32::MAX,
}

impl BuildingTileType {
    pub fn is_belt(&self) -> bool {
        matches!(
            &self,
            Self::BeltUp | Self::BeltDown | Self::BeltLeft | Self::BeltRight
        )
    }

    pub fn next_belt_pos(&self, TilePos { x, y }: TilePos) -> Option<TilePos> {
        use BuildingTileType::*;
        let next_belt_pos = match self {
            BeltUp => TilePos::new(x, y + 1),
            BeltDown if y > 0 => TilePos::new(x, y - 1),
            BeltLeft if x > 0 => TilePos::new(x - 1, y),
            BeltRight => TilePos::new(x + 1, y),
            _ => return None,
        };

        Some(next_belt_pos)
    }

    pub fn next_belt_start(self, next: impl Into<Self>) -> Option<f32> {
        use BuildingTileType::*;

        match (self, next.into()) {
            (BeltDown | BeltUp, BeltLeft | BeltRight) => Some(0.5),
            (BeltLeft | BeltRight, BeltDown | BeltUp) => Some(0.5),
            (x, y) if x == y => Some(0.0),
            _ => None,
        }
    }

    pub fn progress_offset(&self, progress: f32) -> Vec2 {
        use BuildingTileType::*;

        fn lerp(n1: f32, n2: f32, scalar: f32) -> f32 {
            n1 + (n2 - n1) * scalar
        }

        match self {
            BeltUp => Vec2::new(8., lerp(0., 16., progress)),
            BeltDown => Vec2::new(8., lerp(16., 0., progress)),
            BeltLeft => Vec2::new(lerp(16., 0., progress), 8.),
            BeltRight => Vec2::new(lerp(0., 16., progress), 8.),
            _ => panic!("not a belt"),
        }
    }
}

impl From<u32> for BuildingTileType {
    fn from(texture_index: u32) -> Self {
        match texture_index {
            x if x >= BuildingTileType::BeltUp as u32 && x <= BuildingTileType::Chest as u32 => unsafe {
                std::mem::transmute(x)
            },
            _ => Self::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum IoTileType {
    OutputUp = 0,
    OutputDown = 1,
    OutputLeft = 2,
    OutputRight = 3,
    Unknown = u32::MAX,
}

pub fn to_tile_pos(
    world_pos: Vec2,
    tile_size: &TilemapTileSize,
    map_size: &TilemapSize,
    map_transform: &Transform,
) -> Option<TilePos> {
    let x = (world_pos.x - map_transform.translation.x) / tile_size.x;
    let y = (world_pos.y - map_transform.translation.y) / tile_size.y;

    (x > 0. && y > 0. && x < map_size.x as f32 && y < map_size.y as f32)
        .then_some(TilePos::new(x as u32, y as u32))
}

pub fn init_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let terrain_texture = asset_server.load("tilesets/terrain.png");
    let buildings_texture = asset_server.load("tilesets/buildings.png");
    let grid_texture = asset_server.load("tilesets/grid.png");

    let tile_size = TilemapTileSize { x: 16., y: 16. };
    let tilemap_size = TilemapSize { x: 64, y: 64 };

    // Terrain layer

    let mut terrain_storage = TileStorage::empty(tilemap_size);
    let terrain_tilemap = commands.spawn().insert(TerrainLayer).id();

    bevy_ecs_tilemap::helpers::fill_tilemap(
        TileTexture(0),
        tilemap_size,
        TilemapId(terrain_tilemap),
        &mut commands,
        &mut terrain_storage,
    );

    commands
        .entity(terrain_tilemap)
        .insert_bundle(TilemapBundle {
            grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: terrain_storage,
            texture: TilemapTexture(terrain_texture),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..default()
        });

    // Building Layer

    commands
        .spawn_bundle(TilemapBundle {
            grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: TileStorage::empty(tilemap_size),
            texture: TilemapTexture(buildings_texture),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                1.0,
            ),
            ..default()
        })
        .insert(BuildingLayer);

    // Grid layer

    let mut grid_storage = TileStorage::empty(tilemap_size);
    let grid_tilemap = commands.spawn().insert(GridLayer).id();

    bevy_ecs_tilemap::helpers::fill_tilemap(
        TileTexture(0),
        tilemap_size,
        TilemapId(grid_tilemap),
        &mut commands,
        &mut grid_storage,
    );

    commands.entity(grid_tilemap).insert_bundle(TilemapBundle {
        grid_size: TilemapGridSize { x: 16.0, y: 16.0 },
        size: tilemap_size,
        storage: grid_storage,
        texture: TilemapTexture(grid_texture),
        tile_size,
        transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
            &tilemap_size,
            &tile_size,
            2.0,
        ),
        ..default()
    });
}

pub enum MapEvent {
    ToggleGrid,
    ClearBuildings,
}

pub enum GridState {
    Enabled,
    Disabled,
}

impl Default for GridState {
    fn default() -> Self {
        GridState::Enabled
    }
}

impl GridState {
    pub fn toggle(&mut self) {
        *self = match self {
            GridState::Enabled => GridState::Disabled,
            GridState::Disabled => GridState::Enabled,
        }
    }
}

// pub fn clear_buildings(
//     mut commands: Commands,
//     mut map_events: EventReader<MapEvent>,
//     mut map_query: MapQuery,
//     buildings: Query<Entity, With<Building>>,
//     active_map: Res<ActiveMap>,
// ) {
//     if map_events
//         .iter()
//         .any(|e| matches!(e, MapEvent::ClearBuildings))
//     {
//         for building_entity in buildings.iter() {
//             commands.entity(building_entity).despawn();
//         }

//         map_query.despawn_layer_tiles(&mut commands, active_map.map_id, MapLayer::Buildings);

//         if let Some((_, layer)) = map_query.get_layer(active_map.map_id, MapLayer::Buildings) {
//             let chunks = (0..layer.settings.map_size.0)
//                 .flat_map(|x| (0..layer.settings.map_size.1).map(move |y| (x, y)))
//                 .flat_map(|(x, y)| layer.get_chunk(ChunkPos(x, y)))
//                 .collect::<Vec<_>>();

//             for chunk in chunks {
//                 map_query.notify_chunk(chunk);
//             }
//         }
//     }
// }

const MAX_GRID_ZOOM: f32 = 2.;

// pub fn toggle_grid(
//     mut layers: Query<&mut Transform>,
//     mut map_query: MapQuery,
//     mut map_events: EventReader<MapEvent>,
//     mut grid_state: ResMut<GridState>,
//     zoom: Res<Zoom>,
//     active_map: Res<ActiveMap>,
// ) {
//     for _ in map_events
//         .iter()
//         .filter(|e| matches!(e, MapEvent::ToggleGrid))
//     {
//         grid_state.toggle();
//     }

//     if !grid_state.is_changed() && !zoom.is_changed() {
//         return;
//     }

//     if let Some(mut transform) = map_query
//         .get_layer(active_map.map_id, MapLayer::Grid)
//         .and_then(|(e, _)| layers.get_mut(e).ok())
//     {
//         transform.translation.z = match *grid_state {
//             GridState::Enabled if zoom.0 < MAX_GRID_ZOOM => u16::from(MapLayer::Grid) as f32,
//             _ => -10.0,
//         };
//     }
// }

impl From<TileTexture> for BuildingTileType {
    fn from(texture_index: TileTexture) -> Self {
        match texture_index.0 {
            x if x >= BuildingTileType::BeltUp as u32 && x <= BuildingTileType::Chest as u32 => unsafe {
                std::mem::transmute(x)
            },
            _ => Self::Unknown,
        }
    }
}

// impl std::str::FromStr for BuildingTileType {
//     type Err = ParseIntError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let tile_index: u32 = s.parse()?;
//         Ok(tile_index.into())
//     }
// }

impl TryFrom<TileTexture> for TerrainType {
    type Error = ();

    fn try_from(tile: TileTexture) -> Result<Self, ()> {
        match tile.0 {
            x if x >= TerrainType::Grass as u32 && x <= TerrainType::Grass as u32 => {
                Ok(unsafe { std::mem::transmute(x) })
            }
            _ => Err(()),
        }
    }
}

impl From<u32> for IoTileType {
    fn from(texture_index: u32) -> Self {
        match texture_index {
            x if x >= IoTileType::OutputUp as u32 && x <= IoTileType::OutputRight as u32 => unsafe {
                std::mem::transmute(x)
            },
            _ => Self::Unknown,
        }
    }
}
