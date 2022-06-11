use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub const ACTIVE_MAP: u16 = 0;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum MapLayer {
    Terrain = 0,
    Buildings,
    // BuildGuide,
}

impl From<MapLayer> for u16 {
    fn from(layer: MapLayer) -> u16 {
        match layer {
            MapLayer::Terrain => 0,
            MapLayer::Buildings => 1,
        }
    }
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
}

impl BuildingTileType {
    pub fn is_belt(&self) -> bool {
        matches!(
            &self,
            Self::BeltUp | Self::BeltDown | Self::BeltLeft | Self::BeltRight
        )
    }
}

pub fn build_map(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(ACTIVE_MAP, map_entity);

    let map_size = MapSize(3, 3);
    let chunk_size = ChunkSize(16, 16);
    let tile_size = TileSize(16.0, 16.0);

    let layer_settings =
        LayerSettings::new(map_size, chunk_size, tile_size, TextureSize(16.0, 16.0));

    // Build terrain layer
    let (mut layer_builder, layer_entity) =
        LayerBuilder::new(&mut commands, layer_settings, ACTIVE_MAP, MapLayer::Terrain);

    layer_builder.set_all(TileBundle {
        tile: Tile {
            texture_index: TerrainType::Grass as u16,
            ..default()
        },
        ..default()
    });

    let terrain_texture = asset_server.load("terrain.png");
    map_query.build_layer(&mut commands, layer_builder, terrain_texture);

    map.add_layer(&mut commands, MapLayer::Terrain, layer_entity);

    // Build building layer

    let layer_settings =
        LayerSettings::new(map_size, chunk_size, tile_size, TextureSize(16.0 * 4., 16.0));

    let (layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        layer_settings,
        ACTIVE_MAP,
        MapLayer::Buildings,
    );

    let buildings_texture = asset_server.load("buildings.png");
    map_query.build_layer(&mut commands, layer_builder, buildings_texture);

    map.add_layer(&mut commands, MapLayer::Buildings, layer_entity);

    let center = layer_settings.get_pixel_center();
    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());
}

impl TryFrom<Tile> for BuildingTileType {
    type Error = ();

    fn try_from(tile: Tile) -> Result<Self, ()> {
        match tile.texture_index {
            x if x >= BuildingTileType::BeltUp as u16
                && x <= BuildingTileType::BeltRight as u16 =>
            {
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
