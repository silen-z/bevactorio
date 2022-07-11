use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use self::templates::{BuildingTemplates, PlacedBuildingTemplate};
use crate::direction::MapDirection;
use crate::input::MapCursorPos;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::ui::MapInteraction;

pub mod chest;
pub mod mine;
pub mod templates;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Belt,
    Mine,
    Chest,
}

#[derive(Component)]
pub struct Building {
    pub layout: BuildingLayout,
}

#[derive(Component)]
pub struct BuildingTile {
    pub building: Entity,
}

#[derive(Clone, PartialEq, Eq)]
pub enum SelectedTool {
    None,
    Build {
        building: BuildingType,
        direction: MapDirection,
    },
    Buldozer,
}

impl Default for SelectedTool {
    fn default() -> Self {
        SelectedTool::None
    }
}

pub struct BuildRequestedEvent {
    pub building_type: BuildingType,
    pub direction: MapDirection,
    pub tile_pos: TilePos,
}

pub const MAX_BUILDING_SIZE: usize = 9;

#[derive(Clone)]
pub struct BuildingLayout {
    tiles: ArrayVec<(Entity, TilePos, BuildingTileType), MAX_BUILDING_SIZE>,
}

pub struct BuildingBuiltEvent {
    pub building_type: BuildingType,
    pub entity: Entity,
    pub layout: BuildingLayout,
}

pub fn build_building(
    mut commands: Commands,
    mut map_query: MapQuery,
    mut request_events: EventReader<BuildRequestedEvent>,
    mut building_events: EventWriter<BuildingBuiltEvent>,
    templates: Res<BuildingTemplates>,
    active_map: Res<ActiveMap>,
) {
    for event in request_events.iter() {
        let template = templates
            .get(event.building_type)
            .place(event.tile_pos, event.direction);

        if !is_posible_to_build(&template, &mut map_query, &active_map) {
            continue;
        }

        let building_entity = commands.spawn().id();

        let mut tiles = ArrayVec::new();

        for (tile_pos, tile_type) in template.instructions() {
            if let Ok(mine_tile_entity) = map_query.set_tile(
                &mut commands,
                tile_pos,
                Tile {
                    texture_index: tile_type as u16,
                    ..default()
                },
                active_map.map_id,
                MapLayer::Buildings,
            ) {
                commands.entity(mine_tile_entity).insert(BuildingTile {
                    building: building_entity,
                });
                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::Buildings);
                tiles.push((mine_tile_entity, tile_pos, tile_type));
            }
        }

        let layout = BuildingLayout { tiles };

        commands.entity(building_entity).insert(Building {
            layout: layout.clone(),
        });
        building_events.send(BuildingBuiltEvent {
            building_type: event.building_type,
            entity: building_entity,
            layout,
        });
    }
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
                .and_then(|bt| building_query.get(bt.building))
            {
                for (_, tile_pos, _) in building.layout.tiles.iter() {
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
    build_guides: Query<(&TilePos, &mut BuildGuide)>,
    tiles: Query<&Tile>,
    selected_tool: Res<SelectedTool>,
    mouse_pos: Res<MapCursorPos>,
    build_events: EventReader<BuildRequestedEvent>,
    demolish_events: EventReader<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    active_map: Res<ActiveMap>,
    templates: Res<BuildingTemplates>,
) {
    if mouse_pos.is_changed()
        || selected_tool.is_changed()
        || !build_events.is_empty()
        || !demolish_events.is_empty()
    {
        for (tile_pos, _) in build_guides.iter() {
            let _ = map_query.despawn_tile(
                &mut commands,
                *tile_pos,
                active_map.map_id,
                MapLayer::BuildGuide,
            );
            map_query.notify_chunk_for_tile(*tile_pos, active_map.map_id, MapLayer::BuildGuide);
        }

        if let SelectedTool::Build {building, direction} = *selected_tool 
            && let Some(tile_pos) = mouse_pos.0 
            && map_interaction.is_allowed()
        {
            let template = templates.get(building).place(tile_pos, direction);


            let is_belt_edit = |map_query: &mut MapQuery| building == BuildingType::Belt && map_query
                .get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings)
                .ok()
                .and_then(|te| tiles.get(te).ok())
                .map_or(false, |tile| BuildingTileType::from(tile.texture_index).is_belt());

            let guide_color = match is_posible_to_build( &template, &mut map_query, &active_map) {
                true => Color::rgba(0., 1., 0., 0.75),
                false if is_belt_edit(&mut map_query) => Color::rgba(1., 1., 0., 0.75),
                false => Color::rgba(1., 0., 0., 0.75),
            };

            for (tile_pos, building_type) in template.instructions() {
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
}

pub fn highlight_demolition(
    mut commands: Commands,
    mut map_query: MapQuery,
    mouse_pos: Res<MapCursorPos>,
    buildings: Query<&Building>,
    mut tiles: Query<&mut Tile>,
    active_map: Res<ActiveMap>,
    selected_tool: Res<SelectedTool>,
    build_events: EventReader<BuildRequestedEvent>,
    demolish_events: EventReader<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    mut highlighted_buildings: Local<Vec<(TilePos, Entity)>>,
) {
    if mouse_pos.is_changed()
        || selected_tool.is_changed()
        || !build_events.is_empty()
        || !demolish_events.is_empty()
    {
        for (pos, e) in highlighted_buildings.drain(..) {
            if let Ok(mut tile) = tiles.get_mut(e) {
                tile.color = default();
                map_query.notify_chunk_for_tile(pos, active_map.map_id, MapLayer::Buildings);
            }
        }

        if let SelectedTool::Buldozer = *selected_tool 
            && let Some(tile_pos) = mouse_pos.0 
            && map_interaction.is_allowed()
        {
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

            let demolished_building = buildings
                .iter()
                .find_map(|b| b.layout.tiles.iter().any(|(_, pos, _)| *pos == tile_pos).then_some(b));

            if let Some(building) = demolished_building {
                for (e, pos, _) in building.layout.tiles.iter() {
                    if let Ok(mut tile) = tiles.get_mut(*e) {
                        highlighted_buildings.push((*pos, *e));
                        tile.color = Color::RED;
                        map_query.notify_chunk_for_tile(
                            *pos,
                            active_map.map_id,
                            MapLayer::Buildings,
                        );
                    }
                }
            }
        }
    }
}

fn is_posible_to_build(
    template: &PlacedBuildingTemplate,
    map_query: &mut MapQuery,
    active_map: &ActiveMap,
) -> bool {
    template.instructions().all(|(tile_pos, _)| {
        let res = map_query.get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings);
        matches!(res, Err(MapTileError::NonExistent(_)))
    })
}

pub struct UnknownBuildingType;

impl std::str::FromStr for BuildingType {
    type Err = UnknownBuildingType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BuildingType::*;
        let building_type = match s {
            "mine" => Mine,
            _ => return Err(UnknownBuildingType),
        };

        Ok(building_type)
    }
}
