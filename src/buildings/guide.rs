use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::templates::{BuildingTemplate, BuildingTemplates};
use super::{
    is_posible_to_build, BuildRequestedEvent, Building, BuildingType, DemolishEvent, SelectedTool,
};
use crate::input::MapCursorPos;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};
use crate::ui::MapInteraction;

#[derive(Component)]
pub struct BuildGuide;

pub fn show_build_tool(
    mut commands: Commands,
    mut map_query: MapQuery,
    build_guides: Query<(&TilePos, &TileParent), With<BuildGuide>>,
    tiles: Query<&Tile>,
    selected_tool: Res<SelectedTool>,
    mouse_pos: Res<MapCursorPos>,
    build_events: EventReader<BuildRequestedEvent>,
    demolish_events: EventReader<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    active_map: Res<ActiveMap>,
    template_handles: Res<BuildingTemplates>,
    templates: Res<Assets<BuildingTemplate>>,
) {
    if mouse_pos.is_changed()
        || selected_tool.is_changed()
        || !build_events.is_empty()
        || !demolish_events.is_empty()
    {
        for (tile_pos, tile_parent) in build_guides.iter() {
            let _ = map_query.despawn_tile(
                &mut commands,
                *tile_pos,
                active_map.map_id,
                tile_parent.layer_id,
            );
            map_query.notify_chunk_for_tile(*tile_pos, active_map.map_id, tile_parent.layer_id);
        }

        if !map_interaction.is_allowed() {
            return;
        }

        if let SelectedTool::Build {
            building,
            direction,
        } = *selected_tool
        {
            if let Some(tile_pos) = mouse_pos.0 {
                let template = template_handles.get(building);

                let template = templates.get(template).unwrap().place(tile_pos, direction);

                let is_belt_edit = |map_query: &mut MapQuery| {
                    building == BuildingType::Belt
                        && map_query
                            .get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings)
                            .ok()
                            .and_then(|te| tiles.get(te).ok())
                            .map_or(false, |tile| {
                                BuildingTileType::from(tile.texture_index).is_belt()
                            })
                };

                let guide_color = match is_posible_to_build(&template, &mut map_query, &active_map)
                {
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
                        map_query.notify_chunk_for_tile(
                            tile_pos,
                            active_map.map_id,
                            MapLayer::BuildGuide,
                        );
                    }
                }

                for (tile_pos, io_type) in template.io() {
                    info!("{:?} => {}", io_type, io_type as u16);
                    let tile = Tile {
                        texture_index: io_type as u16,
                        ..default()
                    };

                    if let Ok(entity) = map_query.set_tile(
                        &mut commands,
                        tile_pos,
                        tile,
                        active_map.map_id,
                        MapLayer::IoGuide,
                    ) {
                        commands.entity(entity).insert(BuildGuide);
                        map_query.notify_chunk_for_tile(
                            tile_pos,
                            active_map.map_id,
                            MapLayer::IoGuide,
                        );
                    }
                }
            }
        }
    }
}

pub fn show_demo_tool(
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
    let should_update = mouse_pos.is_changed()
        || selected_tool.is_changed()
        || !build_events.is_empty()
        || !demolish_events.is_empty();

    if !should_update {
        return;
    }

    // clean previous guide
    for (pos, e) in highlighted_buildings.drain(..) {
        if let Ok(mut tile) = tiles.get_mut(e) {
            tile.color = default();
            map_query.notify_chunk_for_tile(pos, active_map.map_id, MapLayer::Buildings);
        }
    }

    match (selected_tool.as_ref(), mouse_pos.0) {
        (SelectedTool::Buldozer, Some(tile_pos)) if map_interaction.is_allowed() => {
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
                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::BuildGuide);
            }

            let demolished_building = buildings.iter().find_map(|b| {
                b.layout
                    .tiles
                    .iter()
                    .any(|(_, pos, _)| *pos == tile_pos)
                    .then_some(b)
            });

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
        _ => {}
    }
}
