use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum MapLayer {
    Terrain = 0,
    Grid = 1,
    Buildings = 2,
    // BuildGuide,
}

impl LayerId for MapLayer {}

#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum TerrainType {
    Grass = 0,
}

#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum BuildingTileType {
    BeltUp = 0,
    BeltDown,
    BeltLeft,
    BeltRight,
    Mine,
}

impl BuildingTileType {
    pub fn is_belt(&self) -> bool {
        matches!(
            &self,
            Self::BeltUp | Self::BeltDown | Self::BeltLeft | Self::BeltRight
        )
    }
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
}

impl FromWorld for ActiveMap {
    fn from_world(world: &mut World) -> Self {
        let map_entity = world.spawn().id();

        let asset_server = world.resource::<AssetServer>();

        let terrain_texture = asset_server.load("terrain.png");
        let buildings_texture = asset_server.load("buildings.png");
        let grid_texture = asset_server.load("grid.png");

        let mut dependencies: SystemState<(Commands, MapQuery)> = SystemState::new(world);
        let (mut commands, mut map_query) = dependencies.get_mut(world);

        let map_id = 0u16;
        let mut map = Map::new(map_id, map_entity);

        let map_size = MapSize(3, 3);
        let chunk_size = ChunkSize(16, 16);
        let tile_size = TileSize(16.0, 16.0);

        let layer_settings =
            LayerSettings::new(map_size, chunk_size, tile_size, TextureSize(16.0, 16.0));

        // Build terrain layer
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

        // Build building layer
        let layer_settings = LayerSettings::new(
            map_size,
            chunk_size,
            tile_size,
            TextureSize(16.0 * 5., 16.0),
        );

        let (layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
            &mut commands,
            layer_settings,
            map_id,
            MapLayer::Buildings,
        );

        map_query.build_layer(&mut commands, layer_builder, buildings_texture);

        map.add_layer(&mut commands, MapLayer::Buildings, layer_entity);

        // build grid layer
        let layer_settings =
            LayerSettings::new(map_size, chunk_size, tile_size, TextureSize(16.0, 16.0));

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

        commands
            .entity(layer_entity)
            .insert(Visibility { is_visible: true });

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

impl From<MapLayer> for u16 {
    fn from(layer: MapLayer) -> u16 {
        layer as u16
    }
}

impl TryFrom<Tile> for BuildingTileType {
    type Error = ();

    fn try_from(tile: Tile) -> Result<Self, ()> {
        match tile.texture_index {
            x if x >= BuildingTileType::BeltUp as u16 && x <= BuildingTileType::Mine as u16 => {
                Ok(unsafe { std::mem::transmute(x) })
            }
            _ => Err(()),
        }
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
