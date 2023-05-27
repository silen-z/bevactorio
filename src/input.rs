use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::{BuildRequestedEvent, BuildingType, DemolishEvent, SelectedTool};
use crate::camera::MainCamera;
use crate::map::{to_tile_pos, GridLayer, MapEvent};
use crate::ui::MapInteraction;

pub fn handle_mouse_input(
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<WorldCursorPos>,
    mut build_events: EventWriter<BuildRequestedEvent>,
    mut demolish_events: EventWriter<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    selected_building: Res<SelectedTool>,
    grid_layer_query: Query<(&TilemapSize, &TilemapTileSize, &Transform), With<GridLayer>>,
) {
    let (map_size, tile_size, map_transform) = grid_layer_query.single();

    if let Some(tile_pos) =
        cursor_pos.and_then(|cp| to_tile_pos(cp, tile_size, map_size, map_transform))
    {
        if mouse.pressed(MouseButton::Left) && map_interaction.is_allowed() {
            match *selected_building {
                SelectedTool::Build {
                    building,
                    direction: rotation,
                } => {
                    build_events.send(BuildRequestedEvent {
                        building_type: building,
                        direction: rotation,
                        tile_pos,
                    });
                }

                SelectedTool::Buldozer => {
                    demolish_events.send(DemolishEvent { tile_pos });
                }

                _ => {}
            }
        }
    }
}

pub fn handle_keyboard_input(
    mut key_events: EventReader<KeyboardInput>,
    mut map_events: EventWriter<MapEvent>,
    mut selected_tool: ResMut<SelectedTool>,
) {
    for event in key_events.iter() {
        match event {
            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::G),
                ..
            } => map_events.send(MapEvent::ToggleGrid),

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::C),
                ..
            } => map_events.send(MapEvent::ClearBuildings),

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::M),
                ..
            } => {
                *selected_tool = SelectedTool::Build {
                    building: BuildingType::Mine,
                    direction: default(),
                }
            }

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::B),
                ..
            } => {
                *selected_tool = SelectedTool::Build {
                    building: BuildingType::Belt,
                    direction: default(),
                }
            }

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::D),
                ..
            } => *selected_tool = SelectedTool::Buldozer,

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::R),
                ..
            } => selected_tool.rotate(),

            _ => {}
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut, Debug)]
pub struct WorldCursorPos(pub Option<Vec2>);

pub fn world_cursor_pos(
    windows: Query<(Entity, &Window, Option<&PrimaryWindow>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut world_pos: ResMut<WorldCursorPos>,
) {
    let (camera, camera_transform) = camera_query.single();

    let window = match camera.target {
        RenderTarget::Window(WindowRef::Primary) => {
            let Some((_,window,_)) = windows.iter().find(|(_,_,primary)| primary.is_some()) else {
                return;
            };
            window
        }
        RenderTarget::Window(WindowRef::Entity(e)) => {
            let Some((_,window,_)) = windows.iter().find(|(entity,_,_)| *entity == e) else {
                return;
            };
            window
        }
        _ => return,
    };

    let Some(screen_pos) = window.cursor_position() else {
        return;
    };

    if let Some(ray) = camera.viewport_to_world(camera_transform, screen_pos) {
        world_pos.0 = Some(ray.origin.truncate());
    }
}

#[derive(Resource, Default, Debug)]
pub struct MapCursorPos(pub Option<TilePos>);

pub fn map_cursor_pos(
    mut map_pos: ResMut<MapCursorPos>,
    world_pos: Res<WorldCursorPos>,
    grid_layer_query: Query<(&TilemapSize, &TilemapTileSize, &Transform), With<GridLayer>>,
) {
    let Some(world_pos) = world_pos.0 else {
        return;
    };

    let (map_size, tile_size, map_transform) = grid_layer_query.single();

    let tile_pos = to_tile_pos(world_pos, tile_size, map_size, map_transform);

    if tile_pos != map_pos.0 {
        map_pos.0 = tile_pos;
    }
}
