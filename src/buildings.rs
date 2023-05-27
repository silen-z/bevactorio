use arrayvec::ArrayVec;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use self::templates::{BuildingTemplate, BuildingTemplates, PlacedBuildingTemplate};
use crate::direction::MapDirection;
use crate::map::{BuildingLayer, BuildingTileType, BuildGuideLayer};

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

impl BuildingLayout {
    pub fn contains(&self, tile: &TilePos) -> bool {
        self.tiles.iter().any(|(_, pos, _)| pos == tile)
    }
}

#[derive(Component)]
pub struct BuildingTile {
    pub building: Entity,
}

#[derive(Resource, Default, Clone, PartialEq, Eq)]
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
    pub tiles: ArrayVec<(Entity, TilePos, BuildingTileType), MAX_BUILDING_SIZE>,
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
    mut request_events: EventReader<BuildRequestedEvent>,
    template_handles: Res<BuildingTemplates>,
    templates: Res<Assets<BuildingTemplate>>,
    mut building_layer: Query<&mut TileStorage, With<BuildingLayer>>,
) {
    if request_events.is_empty() {
        return;
    }

    let building_layer = building_layer.single_mut();

    for event in request_events.iter() {
        let template_handle = template_handles.get(event.building_type);

        let template = templates
            .get(&template_handle)
            .unwrap()
            .place(event.tile_pos, event.direction);

        if is_posible_to_build(&template, &building_layer) {
            commands.spawn(BuildingBundle {
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
    mut building_layer: Query<(Entity, &mut TileStorage), With<BuildingLayer>>,
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
    let (building_layer_entity, mut building_layer) = building_layer.single_mut();

    for (building_entity, origin_pos, direction, template_handle, building) in
        changed_buildings.iter()
    {
        // despawn tiles of previous building if it exists
        if let Some(Building { layout }) = building {
            for (entity, tile_pos, _) in &layout.tiles {
                commands.entity(*entity).despawn_recursive();
                building_layer.checked_remove(tile_pos);
            }
        }

        let template = templates
            .get(template_handle)
            .unwrap()
            .place(*origin_pos, *direction);

        let building_entity = commands.spawn_empty().id();
        let mut tiles = ArrayVec::new();

        for (tile_pos, tile_type) in template.instructions() {
            let tile_entity = commands.spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(building_layer_entity),
                texture_index: TileTextureIndex(tile_type as u32),
                ..default()
            }).insert(BuildingTile {
                building: building_entity,
            }).id();

            building_layer.set(&tile_pos, tile_entity);
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
