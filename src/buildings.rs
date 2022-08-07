use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use self::templates::{BuildingTemplate, BuildingTemplates, PlacedBuildingTemplate};
use crate::direction::MapDirection;
use crate::map::{BuildingLayer, BuildingTileType};

pub mod chest;
pub mod mine;
pub mod templates;
pub mod guide;

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Default, PartialEq, Eq)]
pub enum SelectedTool {
    #[default]
    None,
    Build {
        building: BuildingType,
        direction: MapDirection,
    },
    Buldozer,
}

impl SelectedTool {
    pub fn rotate(&mut self) {
        match self {
            SelectedTool::Build { direction, .. } => direction.turn_left(),
            _ => {}
        }
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

#[derive(Bundle)]
struct BuildingBundle {
    building_type: BuildingType,
    origin: TilePos,
    template: Handle<BuildingTemplate>,
    direction: MapDirection,
}

pub fn build_building(
    mut commands: Commands,
    mut building_layer_query: Query<(&TilemapId, &mut TileStorage), With<BuildingLayer>>,
    mut request_events: EventReader<BuildRequestedEvent>,
    template_handles: Res<BuildingTemplates>,
    templates: Res<Assets<BuildingTemplate>>,
    building_layer: Query<&TileStorage, With<BuildingLayer>>,
) {
    let building_layer = building_layer.single();

    for event in request_events.iter() {
        let template_handle = template_handles.get(event.building_type);

        let template = templates
            .get(&template_handle)
            .unwrap()
            .place(event.tile_pos, event.direction);

        if is_posible_to_build(&template, &building_layer) {
            commands.spawn_bundle(BuildingBundle {
                building_type: event.building_type,
                origin: event.tile_pos,
                template: template_handle,
                direction: event.direction,
            });
        }
    }
}

pub fn construct_building(
    mut commands: Commands,
    mut building_layer: Query<&mut TileStorage, With<BuildingLayer>>,
    changed_buildings: Query<
        (
            Entity,
            &TilePos,
            &MapDirection,
            &Handle<BuildingTemplate>,
            Option<&Building>,
        ),
        Changed<Handle<BuildingTemplate>>,
    >,
    templates: Res<Assets<BuildingTemplate>>,
) {
    let mut building_layer = building_layer.single_mut();

    for (building_entity, origin_pos, direction, template_handle, building) in
        changed_buildings.iter()
    {
        if let Some(Building { layout }) = building {
            for (entity, tile_pos, _) in &layout.tiles {
                commands.entity(*entity).despawn_recursive();
                building_layer.set(tile_pos, None);
            }
        }

        let template = templates
            .get(template_handle)
            .unwrap()
            .place(*origin_pos, *direction);

        let mut tiles = ArrayVec::new();

        for (tile_pos, tile_type) in template.instructions() {
            let tile_entity = commands.spawn().insert(BuildingTile {
                building: building_entity,
            }).id();

            building_layer.set(&tile_pos, Some(tile_entity));
            tiles.push((tile_entity, tile_pos, tile_type));
        }

        let layout = BuildingLayout { tiles };

        commands.entity(building_entity).insert(Building { layout });
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
                for (_, tile_pos, _) in building.layout.tiles.iter() {
                    let _ = building_layer.set(tile_pos, None);
                }
                commands.entity(building_entity).despawn();
            } else {
                let _ = building_layer.set(&event.tile_pos, None);
            }
        }
    }
}

#[derive(Component)]
pub struct BuildGuide;

// pub fn update_build_guide(
//     mut commands: Commands,
//     build_guides: Query<(&TilePos, &mut BuildGuide)>,
//     tiles: Query<&TileTexture>,
//     selected_tool: Res<SelectedTool>,
//     mouse_pos: Res<MapCursorPos>,
//     build_events: EventReader<BuildRequestedEvent>,
//     demolish_events: EventReader<DemolishEvent>,
//     map_interaction: Res<MapInteraction>,
//     buildings: Res<BuildingTemplates>,

// ) {

//     if mouse_pos.is_changed()
//         || selected_tool.is_changed()
//         || !build_events.is_empty()
//         || !demolish_events.is_empty()
//     {
//         for (tile_pos, _) in build_guides.iter() {
//             let _ = map_query.despawn_tile(
//                 &mut commands,
//                 *tile_pos,
//                 active_map.map_id,
//                 MapLayer::BuildGuide,
//             );
//             map_query.notify_chunk_for_tile(*tile_pos, active_map.map_id, MapLayer::BuildGuide);
//         }

//         if let SelectedTool::Building(building_type) = *selected_tool
//             && let Some(tile_pos) = mouse_pos.0
//             && map_interaction.is_allowed()
//         {
//             let template = buildings.templates[&building_type].with_origin(tile_pos);

//             let is_belt_edit = |map_query: &mut MapQuery| building_type == BuildingType::Belt && map_query
//                 .get_tile_entity(tile_pos, active_map.map_id, MapLayer::Buildings)
//                 .ok()
//                 .and_then(|te| tiles.get(te).ok())
//                 .map_or(false, |tile| BuildingTileType::from(tile.texture_index).is_belt());

//             let guide_color = match is_posible_to_build( &template.instructions, &mut map_query, &active_map) {
//                 true => Color::rgba(0., 1., 0., 0.75),
//                 false if is_belt_edit(&mut map_query) => Color::rgba(1., 1., 0., 0.75),
//                 false => Color::rgba(1., 0., 0., 0.75),
//             };

//             for (tile_pos, building_type) in template.instructions {
//                 let tile = Tile {
//                     texture_index: building_type as u16,
//                     color: guide_color,
//                     ..default()
//                 };

//                 if let Ok(entity) = map_query.set_tile(
//                     &mut commands,
//                     tile_pos,
//                     tile,
//                     active_map.map_id,
//                     MapLayer::BuildGuide,
//                 ) {
//                     commands.entity(entity).insert(BuildGuide);
//                     map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::BuildGuide);
//                 }
//             }
//         }
//     }
// }

// pub fn highlight_demolition(
//     mut commands: Commands,
//     mut map_query: MapQuery,
//     mouse_pos: Res<MapCursorPos>,
//     buildings: Query<&Building>,
//     mut tiles: Query<&mut Tile>,
//     active_map: Res<ActiveMap>,
//     selected_tool: Res<SelectedTool>,
//     build_events: EventReader<BuildRequestedEvent>,
//     demolish_events: EventReader<DemolishEvent>,
//     map_interaction: Res<MapInteraction>,
//     mut highlighted_buildings: Local<Vec<(TilePos, Entity)>>,
// ) {
//     if mouse_pos.is_changed()
//         || selected_tool.is_changed()
//         || !build_events.is_empty()
//         || !demolish_events.is_empty()
//     {
//         for (pos, e) in highlighted_buildings.drain(..) {
//             if let Ok(mut tile) = tiles.get_mut(e) {
//                 tile.color = default();
//                 map_query.notify_chunk_for_tile(pos, active_map.map_id, MapLayer::Buildings);
//             }
//         }

//         if let SelectedTool::Buldozer = *selected_tool
//             && let Some(tile_pos) = mouse_pos.0
//             && map_interaction.is_allowed()
//         {
//             let tile = Tile {
//                 texture_index: BuildingTileType::Explosion as u16,
//                 ..default()
//             };

//             if let Ok(entity) = map_query.set_tile(
//                 &mut commands,
//                 tile_pos,
//                 tile,
//                 active_map.map_id,
//                 MapLayer::BuildGuide,
//             ) {
//                 commands.entity(entity).insert(BuildGuide);
//                 map_query.notify_chunk_for_tile(
//                     tile_pos,
//                     active_map.map_id,
//                     MapLayer::BuildGuide,
//                 );
//             }

//             let demolished_building = buildings
//                 .iter()
//                 .find_map(|b| b.layout.tiles.iter().any(|(_, pos, _)| *pos == tile_pos).then_some(b));

//             if let Some(building) = demolished_building {
//                 for (e, pos, _) in building.layout.tiles.iter() {
//                     if let Ok(mut tile) = tiles.get_mut(*e) {
//                         highlighted_buildings.push((*pos, *e));
//                         tile.color = Color::RED;
//                         map_query.notify_chunk_for_tile(
//                             *pos,
//                             active_map.map_id,
//                             MapLayer::Buildings,
//                         );
//                     }
//                 }
//             }
//         }
//     }
// }

fn is_posible_to_build(
    template: &PlacedBuildingTemplate,
    building_layer: &TileStorage,
) -> bool {
    template.instructions().all(|(tile_pos, _)| {
        building_layer.get(&tile_pos).is_none()
    })
}

pub struct UnknownBuildingType;

impl std::str::FromStr for BuildingType {
    type Err = UnknownBuildingType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BuildingType::*;
        let building_type = match s {
            "belt" => Belt,
            "mine" => Mine,
            "chest" => Chest,
            _ => return Err(UnknownBuildingType),
        };

        Ok(building_type)
    }
}
