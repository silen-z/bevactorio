use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy::utils::{Hashed, PreHashMap};
use bevy_ecs_tilemap::prelude::*;

use super::{BuildingType, MAX_BUILDING_SIZE};
use crate::map::{BuildingTileType, IoTileType, MapDirection};

pub struct BuildingTemplates {
    pub templates: PreHashMap<BuildingType, BuildingTemplate>,
}

impl BuildingTemplates {
    pub fn get(&self, building: BuildingType) -> &BuildingTemplate {
        &self.templates[&Hashed::new(building)]
    }
}

impl FromWorld for BuildingTemplates {
    fn from_world(_world: &mut World) -> Self {
        let mut loader = tiled::Loader::new();

        let map = loader.load_tmx_map("assets/buildings.tmx").unwrap();

        let mut templates = PreHashMap::default();

        for layer in map.layers() {
            let layer_group = match layer.layer_type() {
                tiled::LayerType::GroupLayer(l) => l,
                _ => continue,
            };

            if let Ok(building_type) = layer.name.parse::<BuildingType>() {
                if let Ok(template) = BuildingTemplate::from_tiled_layer(layer_group) {
                    let hashed_type = Hashed::new(building_type);
                    templates.insert(hashed_type, template);
                }
            }
        }

        BuildingTemplates { templates }
    }
}

pub struct EveryDirection<T> {
    up: T,
    down: T,
    left: T,
    right: T,
}

impl<T> EveryDirection<T> {
    fn map<U>(self, f: impl Fn(T) -> U) -> EveryDirection<U> {
        EveryDirection {
            up: f(self.up),
            down: f(self.down),
            left: f(self.left),
            right: f(self.right),
        }
    }
}

impl<T: Clone> EveryDirection<T> {
    fn all(value: T) -> Self {
        Self {
            up: value.clone(),
            down: value.clone(),
            left: value.clone(),
            right: value.clone(),
        }
    }
}

impl<T: Clone> Clone for EveryDirection<T> {
    fn clone(&self) -> Self {
        Self {
            up: self.up.clone(),
            down: self.down.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl<T> std::ops::Index<MapDirection> for EveryDirection<T> {
    type Output = T;

    fn index(&self, index: MapDirection) -> &Self::Output {
        match index {
            MapDirection::Up => &self.up,
            MapDirection::Down => &self.down,
            MapDirection::Left => &self.left,
            MapDirection::Right => &self.right,
        }
    }
}

type Instructions<T> = ArrayVec<(TilePos, T), MAX_BUILDING_SIZE>;

pub struct BuildingTemplate {
    pub instructions: EveryDirection<Instructions<BuildingTileType>>,
    pub io: EveryDirection<Instructions<IoTileType>>,
}

impl BuildingTemplate {
    pub fn with_origin(&self, origin: TilePos) -> Self {
        let instructions = self
            .instructions
            .clone()
            .map(|direction| adjust_instructions_to_origin(direction, origin));

        Self {
            instructions,
            io: self.io.clone(),
        }
    }

    fn from_tiled_layer(layer_group: tiled::GroupLayer) -> Result<BuildingTemplate, String> {
        let mut base_layer = None;

        for layer in layer_group.layers() {
            match layer.layer_type() {
                tiled::LayerType::TileLayer(l) if layer.name == "base" => {
                    base_layer = dbg!(instructions_from_layer(l))
                }

                _ => continue,
            }
        }

        Ok(BuildingTemplate {
            instructions: EveryDirection::all(base_layer.unwrap()),
            io: EveryDirection::all(ArrayVec::new()),
        })
    }
}

fn adjust_instructions_to_origin<T>(
    instructions: Instructions<T>,
    origin: TilePos,
) -> Instructions<T> {
    instructions
        .into_iter()
        .map(|(tile_pos, tile_type)| {
            let pos = TilePos(origin.0 + tile_pos.0, origin.1 + tile_pos.1);
            (pos, tile_type)
        })
        .collect()
}

fn instructions_from_layer<T: From<u16>>(layer: tiled::TileLayer) -> Option<Instructions<T>> {
    let mut instructions = ArrayVec::new();

    for x in 0..layer.width().unwrap() {
        for y in 0..layer.height().unwrap() {
            if let Some(tile) = layer.get_tile(x as i32, y as i32) {
                instructions.push((
                    TilePos(x, layer.height().unwrap() - y),
                    T::from(tile.id() as u16),
                ));
            }
        }
    }

    Some(instructions)
}
