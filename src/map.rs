use std::num::ParseIntError;

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::Building;
use crate::camera::Zoom;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MapDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Default for MapDirection {
    fn default() -> Self {
        MapDirection::Down
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum MapLayer {
    Terrain = 0,
    Grid = 1,
    Buildings = 2,
    BuildGuide = 3,
}

impl LayerId for MapLayer {}

#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum TerrainType {
    Grass = 0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
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
    Unknown = u16::MAX,
}

impl BuildingTileType {
    pub fn is_belt(&self) -> bool {
        matches!(
            &self,
            Self::BeltUp | Self::BeltDown | Self::BeltLeft | Self::BeltRight
        )
    }

    pub fn next_belt_pos(&self, TilePos(x, y): TilePos) -> Option<TilePos> {
        use BuildingTileType::*;
        let next_belt_pos = match self {
            BeltUp => TilePos(x, y + 1),
            BeltDown if y > 0 => TilePos(x, y - 1),
            BeltLeft if x > 0 => TilePos(x - 1, y),
            BeltRight => TilePos(x + 1, y),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
#[allow(dead_code)]
pub enum IoTileType {
    OutputUp = 0,
    OutputDown = 1,
    OutputLeft = 2,
    OutputRight = 3,
}

pub struct ActiveMap {
    pub map_id: u16,
    pub map_size: MapSize,
    pub map_transform: Transform,
    pub chunk_size: ChunkSize,
    pub tile_size: TileSize,
}

impl ActiveMap {
    pub fn to_world_pos(&self, tile_pos: TilePos) -> Vec2 {
        let x = tile_pos.0 as f32 * self.tile_size.0 + self.map_transform.translation.x;
        let y = tile_pos.1 as f32 * self.tile_size.1 + self.map_transform.translation.y;

        Vec2::new(x, y)
    }

    pub fn to_tile_pos(&self, world_pos: Vec2) -> Option<TilePos> {
        let x = (world_pos.x - self.map_transform.translation.x) / self.tile_size.0;
        let y = (world_pos.y - self.map_transform.translation.y) / self.tile_size.1;

        let map_x_size = (self.map_size.0 * self.chunk_size.0) as f32;
        let map_y_size = (self.map_size.1 * self.chunk_size.1) as f32;

        (x > 0. && y > 0. && x < map_x_size && y < map_y_size)
            .then_some(TilePos(x as u32, y as u32))
    }
}

impl FromWorld for ActiveMap {
    fn from_world(world: &mut World) -> Self {
        let map_entity = world.spawn().id();

        let asset_server = world.resource::<AssetServer>();

        let terrain_texture = asset_server.load("tilesets/terrain.png");
        let terrain_texture_size = TextureSize(16.0, 16.0);

        let buildings_texture = asset_server.load("tilesets/buildings.png");
        let building_texture_size = TextureSize(16.0 * 10., 16.0);

        let grid_texture = asset_server.load("tilesets/grid.png");
        let grid_texture_size = TextureSize(16.0, 16.0);

        let mut dependencies: SystemState<(Commands, MapQuery)> = SystemState::new(world);
        let (mut commands, mut map_query) = dependencies.get_mut(world);

        let map_id = 0u16;
        let mut map = Map::new(map_id, map_entity);

        let map_size = MapSize(1, 1);
        let chunk_size = ChunkSize(64, 64);
        let tile_size = TileSize(16.0, 16.0);

        let layer_settings =
            LayerSettings::new(map_size, chunk_size, tile_size, terrain_texture_size);

        // Build terrain layer
        {
            let (mut layer_builder, layer_entity) =
                LayerBuilder::new(&mut commands, layer_settings, map_id, MapLayer::Terrain);

            layer_builder.set_all(TileBundle {
                tile: Tile {
                    texture_index: TerrainType::Grass as u16,
                    ..default()
                },
                ..default()
            });

            map_query.build_layer(&mut commands, layer_builder, terrain_texture);

            map.add_layer(&mut commands, MapLayer::Terrain, layer_entity);
        }

        // Build building layer
        {
            let layer_settings =
                LayerSettings::new(map_size, chunk_size, tile_size, building_texture_size);

            let (layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
                &mut commands,
                layer_settings,
                map_id,
                MapLayer::Buildings,
            );

            map_query.build_layer(&mut commands, layer_builder, buildings_texture.clone());

            map.add_layer(&mut commands, MapLayer::Buildings, layer_entity);
        }

        // build grid layer
        {
            let layer_settings =
                LayerSettings::new(map_size, chunk_size, tile_size, grid_texture_size);

            let (mut layer_builder, layer_entity) =
                LayerBuilder::new(&mut commands, layer_settings, map_id, MapLayer::Grid);

            layer_builder.set_all(TileBundle {
                tile: Tile {
                    texture_index: 0,
                    ..default()
                },
                ..default()
            });

            map_query.build_layer(&mut commands, layer_builder, grid_texture);

            map.add_layer(&mut commands, MapLayer::Grid, layer_entity);
        }

        // Build building guide layer
        {
            let layer_settings =
                LayerSettings::new(map_size, chunk_size, tile_size, building_texture_size);

            let (layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
                &mut commands,
                layer_settings,
                map_id,
                MapLayer::BuildGuide,
            );

            map_query.build_layer(&mut commands, layer_builder, buildings_texture);

            map.add_layer(&mut commands, MapLayer::BuildGuide, layer_entity);
        }

        dependencies.apply(world);

        let center = layer_settings.get_pixel_center();
        let map_transform = Transform::from_xyz(-center.x, -center.y, 0.0);

        world
            .entity_mut(map_entity)
            .insert(map)
            .insert(map_transform)
            .insert(GlobalTransform::default());

        Self {
            map_id,
            map_size,
            map_transform,
            chunk_size,
            tile_size,
        }
    }
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

pub fn clear_buildings(
    mut commands: Commands,
    mut map_events: EventReader<MapEvent>,
    mut map_query: MapQuery,
    buildings: Query<Entity, With<Building>>,
    active_map: Res<ActiveMap>,
) {
    if map_events
        .iter()
        .any(|e| matches!(e, MapEvent::ClearBuildings))
    {
        for building_entity in buildings.iter() {
            commands.entity(building_entity).despawn();
        }

        map_query.despawn_layer_tiles(&mut commands, active_map.map_id, MapLayer::Buildings);

        if let Some((_, layer)) = map_query.get_layer(active_map.map_id, MapLayer::Buildings) {
            let chunks = (0..layer.settings.map_size.0)
                .flat_map(|x| (0..layer.settings.map_size.1).map(move |y| (x, y)))
                .flat_map(|(x, y)| layer.get_chunk(ChunkPos(x, y)))
                .collect::<Vec<_>>();

            for chunk in chunks {
                map_query.notify_chunk(chunk);
            }
        }
    }
}

const MAX_GRID_ZOOM: f32 = 2.;

pub fn toggle_grid(
    mut layers: Query<&mut Transform>,
    mut map_query: MapQuery,
    mut map_events: EventReader<MapEvent>,
    mut grid_state: ResMut<GridState>,
    zoom: Res<Zoom>,
    active_map: Res<ActiveMap>,
) {
    for _ in map_events
        .iter()
        .filter(|e| matches!(e, MapEvent::ToggleGrid))
    {
        grid_state.toggle();
    }

    if !grid_state.is_changed() && !zoom.is_changed() {
        return;
    }

    if let Some(mut transform) = map_query
        .get_layer(active_map.map_id, MapLayer::Grid)
        .and_then(|(e, _)| layers.get_mut(e).ok())
    {
        transform.translation.z = match *grid_state {
            GridState::Enabled if zoom.0 < MAX_GRID_ZOOM => u16::from(MapLayer::Grid) as f32,
            _ => -10.0,
        };
    }
}

impl From<MapLayer> for u16 {
    fn from(layer: MapLayer) -> u16 {
        layer as u16
    }
}

impl From<u16> for BuildingTileType {
    fn from(texture_index: u16) -> Self {
        match texture_index {
            x if x >= BuildingTileType::BeltUp as u16 && x <= BuildingTileType::Chest as u16 => unsafe {
                std::mem::transmute(x)
            },
            _ => Self::Unknown,
        }
    }
}

impl std::str::FromStr for BuildingTileType {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tile_index: u16 = s.parse()?;
        Ok(tile_index.into())
    }
}

impl TryFrom<Tile> for TerrainType {
    type Error = ();

    fn try_from(tile: Tile) -> Result<Self, ()> {
        match tile.texture_index {
            x if x >= TerrainType::Grass as u16 && x <= TerrainType::Grass as u16 => {
                Ok(unsafe { std::mem::transmute(x) })
            }
            _ => Err(()),
        }
    }
}
