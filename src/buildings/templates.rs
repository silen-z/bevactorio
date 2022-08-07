use arrayvec::ArrayVec;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;

use super::BuildingTileType::*;
use super::{BuildingTileType, BuildingType, MAX_BUILDING_SIZE};

pub struct BuildingTemplates {
    pub templates: HashMap<BuildingType, BuildingTemplate>,
}

impl Default for BuildingTemplates {
    fn default() -> Self {
        let mut templates = HashMap::new();

        templates.insert(BuildingType::Belt, BuildingTemplate::from_single(BeltUp));
        templates.insert(BuildingType::Chest, BuildingTemplate::from_single(Chest));
        // templates.insert(BuildingType::Mine, MINE_TEMPLATE.parse().unwrap());

        Self { templates }
    }
}

pub struct BuildingTemplate {
    pub instructions: ArrayVec<(TilePos, BuildingTileType), MAX_BUILDING_SIZE>,
}

impl BuildingTemplate {
    pub fn from_single(building_type: BuildingTileType) -> Self {
        let mut instructions = ArrayVec::new();
        instructions.push((TilePos::new(0, 0), building_type));

        Self { instructions }
    }

    pub fn with_origin(&self, origin: TilePos) -> Self {
        let instructions = self
            .instructions
            .iter()
            .map(|(tile_pos, tile_type)| {
                let pos = TilePos::new(tile_pos.x + origin.x, tile_pos.y + origin.y);
                (pos, *tile_type)
            })
            .collect();

        Self { instructions }
    }
}

// impl std::str::FromStr for BuildingTemplate {
//     type Err = &'static str;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let lines: Vec<&str> = s.lines().filter(|l| !l.is_empty()).collect();

//         let rows = lines.len() - 1;

//         let width = lines
//             .iter()
//             .map(|l| l.split(' ').count())
//             .max()
//             .ok_or_else(|| "no lines")?;

//         let mut instructions = ArrayVec::new();

//         for (row, line) in lines.into_iter().enumerate() {
//             for (col, tile) in (0..width).zip(line.split(' ').chain(std::iter::once("_"))) {
//                 if tile != "_" {
//                     let tile = tile.parse().unwrap();

//                     instructions.push((TilePos::new(col as u32, (rows - row) as u32), tile));
//                 }
//             }
//         }

//         Ok(Self { instructions })
//     }
// }
