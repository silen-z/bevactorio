use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::input::MapCursorPos;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::ui::UiInteraction;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuildingType {
    Belt,
    Mine,
}

#[derive(Component)]
pub struct Building {
    pub tiles: ArrayVec<(TilePos, Entity), 9>,
}

#[derive(Component)]
pub struct BuildingTile {
    pub building_entity: Entity,
}

#[derive(Clone, PartialEq, Eq)]
pub enum SelectedTool {
    None,
    Building(BuildingType),
    Buldozer,
}

impl Default for SelectedTool {
    fn default() -> Self {
        SelectedTool::None
    }
}

pub struct BuildEvent {
    pub building_type: BuildingType,
    pub tile_pos: TilePos,
}

pub struct DemolishEvent {
    pub tile_pos: TilePos,
}

pub fn demolish_building(
    mut commands: Commands,
    mut map_query: MapQuery,
    mut events: EventReader<DemolishEvent>,
    building_query: Query<(Entity, &Building)>,
    building_tile_query: Query<&BuildingTile>,
    active_map: Res<ActiveMap>,
) {
    for event in events.iter() {
        if let Ok(tile_entity) =
            map_query.get_tile_entity(event.tile_pos, active_map.map_id, MapLayer::Buildings)
        {
            // TODO maybe handle disconnected entities
            if let Ok((building_entity, building)) = building_tile_query
                .get(tile_entity)
                .and_then(|bt| building_query.get(bt.building_entity))
            {
                for (tile_pos, _) in &building.tiles {
                    let _ = map_query.despawn_tile(
                        &mut commands,
                        *tile_pos,
                        active_map.map_id,
                        MapLayer::Buildings,
                    );
                    map_query.notify_chunk_for_tile(
                        *tile_pos,
                        active_map.map_id,
                        MapLayer::Buildings,
                    );
                }
                commands.entity(building_entity).despawn();
            } else {
                let _ = map_query.despawn_tile(
                    &mut commands,
                    event.tile_pos,
                    active_map.map_id,
                    MapLayer::Buildings,
                );
                map_query.notify_chunk_for_tile(
                    event.tile_pos,
                    active_map.map_id,
                    MapLayer::Buildings,
                );
            }
        }
    }
}

#[derive(Component)]
pub struct BuildGuide;

pub fn update_build_guide(
    mut commands: Commands,
    mut map_query: MapQuery,
    mouse_pos: Res<MapCursorPos>,
    build_guides: Query<(&TilePos, &mut BuildGuide)>,
    tiles: Query<&Tile>,
    active_map: Res<ActiveMap>,
    selected_tool: Res<SelectedTool>,
    build_events: EventReader<BuildEvent>,
    demolish_events: EventReader<DemolishEvent>,
    ui_interaction: Res<UiInteraction>,
) {
    if !mouse_pos.is_changed()
        && !selected_tool.is_changed()
        && build_events.is_empty()
        && demolish_events.is_empty()
    {
        return;
    }

    for (tile_pos, _) in build_guides.iter() {
        let _ = map_query.despawn_tile(
            &mut commands,
            *tile_pos,
            active_map.map_id,
            MapLayer::BuildGuide,
        );
        map_query.notify_chunk_for_tile(*tile_pos, active_map.map_id, MapLayer::BuildGuide);
    }

    if ui_interaction.0 {
        return;
    }

    if let Some(tile_pos) = mouse_pos.0 {
        let (selected_building, template) = match *selected_tool {
            SelectedTool::Building(building_type) => {
                (building_type, building_type.template(tile_pos))
            }
            SelectedTool::Buldozer => {
                let tile = Tile {
                    texture_index: BuildingTileType::Explosion as u16,

                    ..default()
                };

                if let Ok(entity) = map_query.set_tile(
                    &mut commands,
                    tile_pos,
                    tile,
                    active_map.map_id,
                    MapLayer::BuildGuide,
                ) {
                    commands.entity(entity).insert(BuildGuide);
                    map_query.notify_chunk_for_tile(
                        tile_pos,
                        active_map.map_id,
                        MapLayer::BuildGuide,
                    );
                }
                return;
            }

            SelectedTool::None => return,
        };

        let possible_to_build = template.positions().all(|tile_pos| {
            let tile = map_query.get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings);
            matches!(tile, Err(MapTileError::NonExistent(..)))
        });

        let is_belt_exception = matches!(selected_building, BuildingType::Belt if map_query
            .get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings)
            .ok()
            .and_then(|te| tiles.get(te).ok())
            .map_or(false, |tile| BuildingTileType::from(tile.texture_index).is_belt())
        );

        let guide_color = match possible_to_build {
            true => Color::rgba(0., 1., 0., 0.75),
            false if is_belt_exception => Color::rgba(1., 1., 0., 0.75),
            false => Color::rgba(1., 0., 0., 0.75),
        };

        for (building_type, tile_pos) in template.instructions() {
            let tile = Tile {
                texture_index: building_type as u16,
                color: guide_color,
                ..default()
            };

            if let Ok(entity) = map_query.set_tile(
                &mut commands,
                tile_pos,
                tile,
                active_map.map_id,
                MapLayer::BuildGuide,
            ) {
                commands.entity(entity).insert(BuildGuide);
                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::BuildGuide);
            }
        }
    }
}

use BuildingTileType::*;

#[rustfmt::skip]
const MINE_TEMPLATE: [Option<BuildingTileType>; 9] = [
    Some(MineBottomLeft),  Some(MineBottomRight), None,
    Some(MineTopLeft),     Some(MineTopRight),    None,
    None,                  None,                  None,
];

impl BuildingType {
    pub fn template(&self, origin: TilePos) -> BuildingTemplate {
        let template = match self {
            BuildingType::Belt => return BuildingTemplate::from_single(BeltUp, origin),
            BuildingType::Mine => MINE_TEMPLATE,
        };

        BuildingTemplate::from_static(&template, origin)
    }
}

pub struct BuildingTemplate {
    pub tiles: ArrayVec<(BuildingTileType, TilePos), 9>,
}

impl BuildingTemplate {
    pub fn from_single(building_type: BuildingTileType, origin: TilePos) -> Self {
        let mut tiles = ArrayVec::new();
        tiles.push((building_type, origin));
        Self { tiles }
    }

    pub fn from_static(template: &[Option<BuildingTileType>], origin: TilePos) -> Self {
        let tiles = template
            .into_iter()
            .enumerate()
            .flat_map(move |(i, tile)| {
                tile.map(|t| {
                    let tile_pos = TilePos(origin.0 + i as u32 % 3, origin.1 + i as u32 / 3);
                    (t, tile_pos)
                })
            })
            .collect();

        Self { tiles }
    }

    pub fn instructions(self) -> impl Iterator<Item = (BuildingTileType, TilePos)> {
        self.tiles.into_iter()
    }

    pub fn positions(&self) -> impl Iterator<Item = TilePos> + '_ {
        self.tiles.iter().copied().map(|(_, tile_pos)| tile_pos)
    }
}
