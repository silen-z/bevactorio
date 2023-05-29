use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::Building;

#[derive(Component)]
pub struct TerrainLayer;

#[derive(Component)]
pub struct BuildingLayer;

#[derive(Component)]
pub struct BuildGuideLayer;

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

impl From<BuildingTileType> for TileTextureIndex {
    fn from(value: BuildingTileType) -> Self {
        TileTextureIndex(value as u32)
    }
}

pub fn to_tile_pos(
    world_pos: Vec2,
    tile_size: &TilemapTileSize,
    map_size: &TilemapSize,
    map_transform: &Transform,
) -> Option<TilePos> {
    let x = (world_pos.x - map_transform.translation.x + (tile_size.x * 0.5)) / tile_size.x;
    let y = (world_pos.y - map_transform.translation.y + (tile_size.y * 0.5)) / tile_size.y;

    if x < 0. && y < 0. && x > map_size.x as f32 && y > map_size.y as f32 {
        return None;
    }

    Some(TilePos::new(x as u32, y as u32))
}

pub const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16., y: 16. };
pub const TILEMAP_SIZE: TilemapSize = TilemapSize { x: 64, y: 64 };

pub const GRID_SIZE: TilemapGridSize = TilemapGridSize { x: 16., y: 16. };

pub fn init_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let terrain_texture = asset_server.load("tilesets/terrain.png");
    let buildings_texture = asset_server.load("tilesets/buildings.png");

    // Terrain layer

    let mut terrain_storage = TileStorage::empty(TILEMAP_SIZE);
    let terrain_tilemap = commands.spawn(TerrainLayer).id();

    bevy_ecs_tilemap::helpers::filling::fill_tilemap(
        TileTextureIndex(0),
        TILEMAP_SIZE,
        TilemapId(terrain_tilemap),
        &mut commands,
        &mut terrain_storage,
    );

    commands.entity(terrain_tilemap).insert(TilemapBundle {
        grid_size: GRID_SIZE,
        size: TILEMAP_SIZE,
        storage: terrain_storage,
        texture: TilemapTexture::Single(terrain_texture),
        tile_size: TILE_SIZE,
        transform: bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform(
            &TILEMAP_SIZE,
            &GRID_SIZE,
            &TilemapType::Square,
            0.0,
        ),
        ..default()
    });

    // Building Layer

    commands
        .spawn(TilemapBundle {
            grid_size: GRID_SIZE,
            size: TILEMAP_SIZE,
            storage: TileStorage::empty(TILEMAP_SIZE),
            texture: TilemapTexture::Single(buildings_texture.clone()),
            tile_size: TILE_SIZE,
            transform: bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform(
                &TILEMAP_SIZE,
                &GRID_SIZE,
                &TilemapType::Square,
                1.0,
            ),
            ..default()
        })
        .insert(BuildingLayer);

    // Build guide layer

    commands
        .spawn(TilemapBundle {
            grid_size: GRID_SIZE,
            size: TILEMAP_SIZE,
            storage: TileStorage::empty(TILEMAP_SIZE),
            texture: TilemapTexture::Single(buildings_texture),
            tile_size: TILE_SIZE,
            transform: bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform(
                &TILEMAP_SIZE,
                &GRID_SIZE,
                &TilemapType::Square,
                2.0,
            ),
            ..default()
        })
        .insert(BuildGuideLayer);
}

pub enum MapEvent {
    ToggleGrid,
    ClearBuildings,
}

pub fn clear_buildings(
    mut commands: Commands,
    buildings: Query<(Entity, &Building)>,
    mut building_tilemap: Query<&mut TileStorage, With<BuildingLayer>>,
) {
    let Ok(mut building_tilemap) = building_tilemap.get_single_mut() else {
            error!("no building layer");
            return;
        };

    for (building_entity, building) in buildings.iter() {
        for (entity, tile_pos, _) in &building.layout.tiles {
            commands.entity(*entity).despawn_recursive();
            building_tilemap.checked_remove(tile_pos);
        }

        commands.entity(building_entity).despawn();
    }
}

pub fn should_clear_buildings(mut map_events: EventReader<MapEvent>) -> bool {
    map_events
        .iter()
        .any(|e| matches!(e, MapEvent::ClearBuildings))
}

impl From<TileTextureIndex> for BuildingTileType {
    fn from(texture_index: TileTextureIndex) -> Self {
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

impl TryFrom<TileTextureIndex> for TerrainType {
    type Error = ();

    fn try_from(tile: TileTextureIndex) -> Result<Self, ()> {
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
