use arrayvec::ArrayVec;
use bevy::asset::LoadedAsset;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::{Hashed, PreHashMap};
use bevy_ecs_tilemap::prelude::*;

use super::{BuildingType, MAX_BUILDING_SIZE};
use crate::direction::{Directional, MapDirection};
use crate::map::{BuildingTileType, IoTileType};

type Instructions<T> = ArrayVec<(TilePos, T), MAX_BUILDING_SIZE>;

pub struct BuildingTemplate {
    pub instructions: Directional<Instructions<BuildingTileType>>,
    pub io: Directional<Instructions<IoTileType>>,
}

impl BuildingTemplate {
    pub fn place(&self, origin: TilePos, direction: MapDirection) -> PlacedBuildingTemplate {
        PlacedBuildingTemplate {
            template: self,
            origin,
            direction,
        }
    }

    fn from_tilemap(map: tiled::Map) -> Result<BuildingTemplate, String> {
        let mut base_layer = None;

        for layer in map.layers() {
            match layer.layer_type() {
                tiled::LayerType::TileLayer(l) if layer.name == "base" => {
                    base_layer = instructions_from_layer(l)
                }

                _ => continue,
            }
        }

        Ok(BuildingTemplate {
            instructions: Directional::all(base_layer.unwrap()),
            io: Directional::all(ArrayVec::new()),
        })
    }
}

fn instructions_from_layer<T: From<u16>>(layer: tiled::TileLayer) -> Option<Instructions<T>> {
    let width = layer.width().unwrap();
    let height = layer.height().unwrap();

    let mut instructions = ArrayVec::new();

    for x in 0..width {
        for y in 0..height {
            if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                let tile_pos = TilePos(x, height - 1 - y);
                let tile = tile.id() as u16;
                instructions.push((tile_pos, tile.into()));
            }
        }
    }

    Some(instructions)
}

pub struct PlacedBuildingTemplate<'t> {
    template: &'t BuildingTemplate,
    origin: TilePos,
    direction: MapDirection,
}

impl PlacedBuildingTemplate<'_> {
    pub fn instructions(&self) -> impl Iterator<Item = (TilePos, BuildingTileType)> + '_ {
        self.template.instructions[self.direction]
            .iter()
            .map(|(tile_pos, tile_type)| {
                let pos = TilePos(self.origin.0 + tile_pos.0, self.origin.1 + tile_pos.1);
                (pos, *tile_type)
            })
    }
}

#[derive(Default)]
pub struct BuildingTemplates {
    pub templates: PreHashMap<BuildingType, BuildingTemplate>,
}

impl BuildingTemplates {
    fn register(&mut self, building_type: BuildingType, template: BuildingTemplate) {
        self.templates.insert(Hashed::new(building_type), template);
    }

    pub fn get(&self, building: BuildingType) -> &BuildingTemplate {
        &self.templates[&Hashed::new(building)]
    }
}

#[derive(TypeUuid)]
#[uuid = "a5bf35d0-f823-4a41-8e54-dd1bd4ed0acd"]
pub struct BuildingTilemap {
    building_type: BuildingType,
    tilemap: tiled::Map,
}

pub struct BuildingTilemapLoader;

impl bevy::asset::AssetLoader for BuildingTilemapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut loader = tiled::Loader::new();

            let path = load_context.path();

            let building_type: BuildingType = path
                .file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".building.tmx"))
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow::anyhow!("invalid building"))?;

            let tilemap = loader.load_tmx_map_from(std::io::BufReader::new(bytes), path)?;

            load_context.set_default_asset(LoadedAsset::new(BuildingTilemap {
                building_type,
                tilemap,
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["building.tmx"];
        EXTENSIONS
    }
}

pub struct BuildingTilemapAssets(Vec<HandleUntyped>);

pub fn load_building_templates(mut commands: Commands, assets: Res<AssetServer>) {
    if let Ok(handles) = assets.load_folder("buildings") {
        commands.insert_resource(BuildingTilemapAssets(handles));
    }
}

pub fn register_building_templates(
    mut assets: ResMut<Assets<BuildingTilemap>>,
    mut asset_events: EventReader<AssetEvent<BuildingTilemap>>,
    mut building_templates: ResMut<BuildingTemplates>,
) {
    for event in asset_events.iter() {
        if let AssetEvent::Created { handle } = event {
            let BuildingTilemap {
                building_type,
                tilemap,
            } = assets.remove(handle).unwrap();

            let Ok(template) = BuildingTemplate::from_tilemap(tilemap) else {
                warn!("unable to load building");
                continue;
            };

            building_templates.register(building_type, template);
        }
    }
}
