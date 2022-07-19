use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use self::templates::{BuildingTemplate, BuildingTemplates, PlacedBuildingTemplate};
use crate::direction::MapDirection;
use crate::map::{ActiveMap, BuildingTileType, MapLayer};

pub mod chest;
pub mod guide;
pub mod mine;
pub mod templates;

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
    mut map_query: MapQuery,
    mut request_events: EventReader<BuildRequestedEvent>,
    template_handles: Res<BuildingTemplates>,
    templates: Res<Assets<BuildingTemplate>>,
    active_map: Res<ActiveMap>,
) {
    for event in request_events.iter() {
        let template_handle = template_handles.get(event.building_type);
        // .place(event.tile_pos, event.direction);

        let template = templates
            .get(template_handle.clone())
            .unwrap()
            .place(event.tile_pos, event.direction);

        if is_posible_to_build(&template, &mut map_query, &active_map) {
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
    mut map_query: MapQuery,
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
    active_map: Res<ActiveMap>,
) {
    for (building_entity, origin_pos, direction, template_handle, building) in
        changed_buildings.iter()
    {
        info!("test");

        if let Some(Building { layout }) = building {
            for (_, tile_pos, _) in &layout.tiles {
                let _ = map_query.despawn_tile(
                    &mut commands,
                    *tile_pos,
                    active_map.map_id,
                    MapLayer::Buildings,
                );
            }
        }

        let template = templates
            .get(template_handle)
            .unwrap()
            .place(*origin_pos, *direction);

        let mut tiles = ArrayVec::new();

        for (tile_pos, tile_type) in template.instructions() {
            if let Ok(building_tile_entity) = map_query.set_tile(
                &mut commands,
                tile_pos,
                Tile {
                    texture_index: tile_type as u16,
                    ..default()
                },
                active_map.map_id,
                MapLayer::Buildings,
            ) {
                commands.entity(building_tile_entity).insert(BuildingTile {
                    building: building_entity,
                });
                map_query.notify_chunk_for_tile(tile_pos, active_map.map_id, MapLayer::Buildings);
                tiles.push((building_tile_entity, tile_pos, tile_type));
            }
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
            "belt" => Belt,
            "mine" => Mine,
            "chest" => Chest,
            _ => return Err(UnknownBuildingType),
        };

        Ok(building_type)
    }
}
