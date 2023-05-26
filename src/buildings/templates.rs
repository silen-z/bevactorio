use arrayvec::ArrayVec;
use bevy::asset::LoadedAsset;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::{Hashed, PreHashMap};
use bevy_ecs_tilemap::prelude::*;
use tiled::LayerType;

use super::{BuildingType, MAX_BUILDING_SIZE};
use crate::direction::{Directional, MapDirection};
use crate::map::{BuildingTileType, IoTileType};

type Instructions<T> = ArrayVec<(TilePos, T), MAX_BUILDING_SIZE>;

#[derive(TypeUuid)]
#[uuid = "a5bf35d0-f823-4a41-8e54-dd1bd4ed0acd"]
pub struct BuildingTemplate {
    pub building_type: BuildingType,
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

    fn from_tilemap(
        building_type: BuildingType,
        map: tiled::Map,
    ) -> anyhow::Result<BuildingTemplate> {
        Ok(BuildingTemplate {
            building_type,
            instructions: Directional {
                up: get_layer(&map, "base", MapDirection::Up).unwrap(),
                down: get_layer(&map, "base", MapDirection::Down).unwrap(),
                left: get_layer(&map, "base", MapDirection::Left).unwrap(),
                right: get_layer(&map, "base", MapDirection::Right).unwrap(),
            },
            io: Directional {
                up: get_layer(&map, "io", MapDirection::Up).unwrap_or_default(),
                down: get_layer(&map, "io", MapDirection::Down).unwrap_or_default(),
                left: get_layer(&map, "io", MapDirection::Left).unwrap_or_default(),
                right: get_layer(&map, "io", MapDirection::Right).unwrap_or_default(),
            },
        })
    }
}

fn get_layer<T: From<u32>>(
    map: &tiled::Map,
    layer_name: &str,
    direction: MapDirection,
) -> Option<Instructions<T>> {
    let direction_group = map.layers().find_map(|layer| match layer.layer_type() {
        LayerType::GroupLayer(l) if direction == layer.name => Some(l),
        _ => None,
    });

    let by_name = |l: &tiled::Layer| l.name == layer_name;

    let layer = direction_group
        .and_then(|g| g.layers().find(by_name))
        .or(map.layers().find(by_name))?;

    match layer.layer_type() {
        tiled::LayerType::TileLayer(l) => instructions_from_layer(l),
        _ => None,
    }
}

fn instructions_from_layer<T: From<u32>>(layer: tiled::TileLayer) -> Option<Instructions<T>> {
    let width = layer.width().unwrap();
    let height = layer.height().unwrap();

    let mut instructions = ArrayVec::new();

    for x in 0..width {
        for y in 0..height {
            if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                let tile_pos = TilePos::new(x, height - 1 - y);
                instructions.push((tile_pos, tile.id().into()));
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
                let pos = TilePos::new(self.origin.x + tile_pos.x, self.origin.y + tile_pos.y);
                (pos, *tile_type)
            })
    }   
    
    pub fn io(&self) -> impl Iterator<Item = (TilePos, IoTileType)> + '_ {
        self.template.io[self.direction]
            .iter()
            .map(|(tile_pos, tile_type)| {
                let pos = TilePos::new(self.origin.x + tile_pos.x, self.origin.y + tile_pos.y);
                (pos, *tile_type)
            })
    }
}

#[derive(Resource, Default)]
pub struct BuildingTemplates {
    templates: PreHashMap<BuildingType, Handle<BuildingTemplate>>,
    loading_handles: Vec<HandleUntyped>,
}

impl BuildingTemplates {
    fn register(&mut self, building_type: BuildingType, template: Handle<BuildingTemplate>) {
        self.templates.insert(Hashed::new(building_type), template);
    }

    pub fn get(&self, building: BuildingType) -> Handle<BuildingTemplate> {
        self.templates[&Hashed::new(building)].clone()
    }
}

pub struct BuildingTemplateLoader;

impl bevy::asset::AssetLoader for BuildingTemplateLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut loader = tiled::Loader::new();

            let path = load_context.path();
            let tilemap = loader.load_tmx_map_from(std::io::BufReader::new(bytes), path)?;

            let building_type = path
                .file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".building.tmx"))
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| anyhow::anyhow!("unknown building {}", path.display()))?;

            let template = BuildingTemplate::from_tilemap(building_type, tilemap)?;

            load_context.set_default_asset(LoadedAsset::new(template));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["building.tmx"];
        EXTENSIONS
    }
}

pub fn load_building_templates(assets: Res<AssetServer>, mut templates: ResMut<BuildingTemplates>) {
    match assets.load_folder("buildings") {
        Ok(handles) => {
            templates.loading_handles.extend(handles);
        }
        Err(e) => warn!("couldn't load building templates: {}", e),
    }
}

pub fn register_building_templates(
    templates: Res<Assets<BuildingTemplate>>,
    mut asset_events: EventReader<AssetEvent<BuildingTemplate>>,
    mut building_templates: ResMut<BuildingTemplates>,
    mut buildings: Query<&mut Handle<BuildingTemplate>>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(index) = building_templates
                    .loading_handles
                    .iter()
                    .position(|h| h.id() == handle.id())
                {
                    let template = templates.get(handle).unwrap();
                    let handle = building_templates.loading_handles.swap_remove(index);
                    building_templates.register(template.building_type, handle.typed());
                }
            }
            AssetEvent::Modified { handle } => {
                for building_handle in buildings.iter_mut() {
                    if &*building_handle == handle {
                        building_handle.into_inner();
                    }
                }
            }
            _ => {}
        }
    }
}
