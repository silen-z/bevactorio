use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use self::templates::BuildingTemplates;
use crate::input::MapCursorPos;
use crate::map::{BuildingLayer, BuildingTileType, BuildGuideLayer};
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

#[derive(Resource, Clone, PartialEq, Eq)]
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

pub struct BuildRequestedEvent {
    pub building_type: BuildingType,
    pub tile_pos: TilePos,
}

pub const MAX_BUILDING_SIZE: usize = 9;

#[derive(Clone)]
pub struct BuildingLayout {
    pub tiles: ArrayVec<(Entity, TilePos, BuildingTileType), MAX_BUILDING_SIZE>,
}

impl BuildingLayout {
    pub fn contains(&self, tile: &TilePos) -> bool {
        self.tiles.iter().any(|(_, pos, _)| pos == tile)
    }
}

pub struct BuildingBuiltEvent {
    pub building_type: BuildingType,
    pub entity: Entity,
    pub layout: BuildingLayout,
}

pub fn build_building(
    mut commands: Commands,
    mut building_layer_query: Query<(Entity, &mut TileStorage), With<BuildingLayer>>,
    mut request_events: EventReader<BuildRequestedEvent>,
    mut building_events: EventWriter<BuildingBuiltEvent>,
    buildings: Res<BuildingTemplates>,
) {
    let (tilemap_entity, mut building_layer) = building_layer_query.single_mut();

    for event in request_events.iter() {
        let template = buildings.templates[&event.building_type].with_origin(event.tile_pos);

        if !is_posible_to_build(&template.instructions, &building_layer) {
            continue;
        }

        let building_entity = commands.spawn_empty().id();

        let mut tiles = ArrayVec::new();

        for (tile_pos, tile_type) in template.instructions {
            let building_tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(tile_type as u32),
                    ..default()
                })
                .insert(BuildingTile {
                    building: building_entity,
                })
                .id();

            building_layer.set(&tile_pos, building_tile_entity);
            tiles.push((building_tile_entity, tile_pos, tile_type));
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
    mut events: EventReader<DemolishEvent>,
    building_query: Query<(Entity, &Building)>,
    building_tile_query: Query<&BuildingTile>,
    mut building_layer_query: Query<&mut TileStorage, With<BuildingLayer>>,
) {
    let mut building_layer = building_layer_query.single_mut();

    for event in events.iter() {
        if let Some(tile_entity) = building_layer.get(&event.tile_pos) {
            // TODO maybe handle disconnected entities
            if let Ok((building_entity, building)) = building_tile_query
                .get(tile_entity)
                .and_then(|bt| building_query.get(bt.building))
            {
                for (e, tile_pos, _) in building.layout.tiles.iter() {
                    commands.entity(*e).despawn_recursive();
                    building_layer.checked_remove(tile_pos);
                }
                commands.entity(building_entity).despawn();
            } else {
                building_layer.checked_remove(&event.tile_pos);
            }
        }
    }
}

#[derive(Component)]
pub struct BuildGuide;

pub fn update_build_guide(
    mut commands: Commands,
    build_guides: Query<(Entity, &TilePos), With<BuildGuide>>,
    tiles: Query<&TileTextureIndex>,
    selected_tool: Res<SelectedTool>,
    mouse_pos: Res<MapCursorPos>,
    build_events: EventReader<BuildRequestedEvent>,
    demolish_events: EventReader<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    buildings: Res<BuildingTemplates>,
    mut guide_tiles: Query<(Entity, &mut TileStorage), With<BuildGuideLayer>>,
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

        // remove previous build guide
        for (tile_entity, tile_pos) in build_guides.iter() {
            commands.entity(tile_entity).despawn_recursive();
            guide_tiles.checked_remove(tile_pos);
        }

        if let SelectedTool::Building(building_type) = *selected_tool
            && let Some(tile_pos) = mouse_pos.0
            && map_interaction.is_allowed()
        {
            let template = buildings.templates[&building_type].with_origin(tile_pos);

            let is_belt_edit = || building_type == BuildingType::Belt && guide_tiles.checked_get(&tile_pos)
                .and_then(|te| tiles.get(te).ok())
                .map_or(false, |tile| BuildingTileType::from(*tile).is_belt());

            let guide_color = match is_posible_to_build( &template.instructions, &guide_tiles) {
                true => Color::rgba(0., 1., 0., 0.75),
                false if is_belt_edit() => Color::rgba(1., 1., 0., 0.75),
                false => Color::rgba(1., 0., 0., 0.75),
            };

            commands.entity(guide_entity).with_children(|parent| {
                for (tile_pos, building_type) in template.instructions {
                    let tile = parent.spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(parent.parent_entity()),
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
    mouse_pos: Res<MapCursorPos>,
    buildings: Query<&Building>,
    selected_tool: Res<SelectedTool>,
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

        if let SelectedTool::Buldozer = *selected_tool
            && let Some(tile_pos) = mouse_pos.0
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

fn is_posible_to_build(
    instructions: &[(TilePos, BuildingTileType)],
    building_layer: &TileStorage,
) -> bool {
    instructions
        .iter()
        .all(|(tile_pos, _)| building_layer.get(tile_pos).is_none())
}
