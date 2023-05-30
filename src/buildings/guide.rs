use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::templates::{BuildingTemplate, BuildingTemplates};
use super::{
    is_posible_to_build, BuildRequestedEvent, Building, BuildingType, DemolishEvent, Tool, BuildTool,
};
use crate::input::GameCursor;
use crate::map::{BuildingTileType, BuildGuideLayer};
use crate::ui::MapInteraction;

#[derive(Component)]
pub struct BuildGuide;

pub fn update_build_guide(
    mut commands: Commands,
    build_guides: Query<(Entity, &TilePos), With<BuildGuide>>,
    tiles: Query<&TileTextureIndex>,
    selected_tool: Res<Tool>,
    mouse_pos: Res<GameCursor>,
    build_events: EventReader<BuildRequestedEvent>,
    demolish_events: EventReader<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    buildings: Res<BuildingTemplates>,
    templates: Res<Assets<BuildingTemplate>>,
    mut guide_tiles: Query<&mut TileStorage, With<BuildGuideLayer>>,
    guide_tilemap: Query<Entity, With<BuildGuideLayer>>,
) {
    if mouse_pos.is_changed()
        || selected_tool.is_changed()
        || !build_events.is_empty()
        || !demolish_events.is_empty()
    {
        let Ok(mut guide_tiles) = guide_tiles.get_single_mut() else {
            warn!("no building layer");
            return;
        };

        // remove previous build guide
        for (tile_entity, tile_pos) in build_guides.iter() {
            commands.entity(tile_entity).despawn_recursive();
            guide_tiles.checked_remove(tile_pos);
        }

        if let Tool::Build(BuildTool {building, direction }) = *selected_tool
            && let Some(tile_pos) = mouse_pos.tile_pos
            && map_interaction.is_allowed()
        {
            let template_handle = buildings.get(building);

            let template = templates
                .get(&template_handle)
                .unwrap()
                .place(tile_pos, direction);

            let is_belt_edit = || building == BuildingType::Belt && guide_tiles.checked_get(&tile_pos)
                .and_then(|te| tiles.get(te).ok())
                .map_or(false, |tile| BuildingTileType::from(*tile).is_belt());

            let guide_color = match is_posible_to_build(&template, &guide_tiles) {
                true => Color::rgba(0., 1., 0., 0.75),
                false if is_belt_edit() => Color::rgba(1., 1., 0., 0.75),
                false => Color::rgba(1., 0., 0., 0.75),
            };

            let guide_tilemap_entity = guide_tilemap.single();

            commands.entity(guide_tilemap_entity).with_children(|parent| {
                for (tile_pos, building_type) in template.instructions() {
                    let tile = parent.spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(guide_tilemap_entity),
                        texture_index: building_type.into(),
                        color: guide_color.into(),
                        ..default()
                    }).insert(BuildGuide).id();                
    
                    guide_tiles.checked_set(&tile_pos, tile);
                }
            });       
        }
    }
}

pub fn highlight_demolition(
    mut commands: Commands,
    mouse_pos: Res<GameCursor>,
    buildings: Query<&Building>,
    selected_tool: Res<Tool>,
    build_events: EventReader<BuildRequestedEvent>,
    demolish_events: EventReader<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    mut building_tiles: Query<&mut TileColor>,
    mut guide_tiles: Query<(Entity, &mut TileStorage), With<BuildGuideLayer>>,
    mut highlighted_buildings: Local<Vec<Entity>>,
) {
    if mouse_pos.is_changed()
        || selected_tool.is_changed()
        || !build_events.is_empty()
        || !demolish_events.is_empty()
    {
        let Ok((guide_entity, mut guide_tiles)) = guide_tiles.get_single_mut() else {
            warn!("no building layer");
            return;
        };

        // clear previously highlighted building tiles
        for e in highlighted_buildings.drain(..) {
            if let Ok(mut tile) = building_tiles.get_mut(e) {
                *tile = default();
            }
        }

        if let Tool::Buldozer = *selected_tool
            && let Some(tile_pos) = mouse_pos.tile_pos
            && map_interaction.is_allowed()
        {
            commands.entity(guide_entity).with_children(|parent| {
                let entity = parent.spawn(TileBundle {
                    tilemap_id: TilemapId(parent.parent_entity()),
                    texture_index: BuildingTileType::Explosion.into(),
                    position: tile_pos,
                    ..default()
                }).insert(BuildGuide).id();
                
                guide_tiles.checked_set(&tile_pos, entity);

                let Some(building) = buildings.iter().find(|b| b.layout.contains(&tile_pos)) else {
                    return;
                };

                // highlight tiles of a building about to be demolished
                for (e, _ , _) in building.layout.tiles.iter() {
                    if let Ok(mut tile) = building_tiles.get_mut(*e) {
                        highlighted_buildings.push(*e);
                        *tile = Color::RED.into();
                    }
                }
    
            });
        }
    }
}
