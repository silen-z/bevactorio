use bevy::input::keyboard::KeyboardInput;
use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::{BuildEvent, BuildingType, DemolishEvent, SelectedTool};
use crate::camera::MainCamera;
use crate::map::{ActiveMap, MapEvent};
use crate::ui::MapInteraction;

pub fn handle_mouse_input(
    mouse: Res<Input<MouseButton>>,
    map_pos: Res<WorldCursorPos>,
    mut build_events: EventWriter<BuildEvent>,
    mut demolish_events: EventWriter<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    selected_building: Res<SelectedTool>,
    active_map: Res<ActiveMap>,
) {
    if let Some(tile_pos) = map_pos.and_then(|cursor_pos| active_map.to_tile_pos(cursor_pos)) {
        if mouse.pressed(MouseButton::Left) && map_interaction.is_allowed() {
            match *selected_building {
                SelectedTool::Building(building_type) => {
                    build_events.send(BuildEvent {
                        building_type,
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
                state: ElementState::Pressed,
                key_code: Some(KeyCode::G),
                ..
            } => map_events.send(MapEvent::ToggleGrid),

            KeyboardInput {
                state: ElementState::Pressed,
                key_code: Some(KeyCode::C),
                ..
            } => map_events.send(MapEvent::ClearBuildings),

            KeyboardInput {
                state: ElementState::Pressed,
                key_code: Some(KeyCode::M),
                ..
            } => *selected_tool = SelectedTool::Building(BuildingType::Mine),

            KeyboardInput {
                state: ElementState::Pressed,
                key_code: Some(KeyCode::B),
                ..
            } => *selected_tool = SelectedTool::Building(BuildingType::Belt),

            KeyboardInput {
                state: ElementState::Pressed,
                key_code: Some(KeyCode::D),
                ..
            } => *selected_tool = SelectedTool::Buldozer,

            _ => {}
        }
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct WorldCursorPos(pub Option<Vec2>);

pub fn world_cursor_pos(
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut world_pos: ResMut<WorldCursorPos>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_query.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = match camera.target {
        RenderTarget::Window(id) => wnds.get(id).unwrap(),
        _ => wnds.get_primary().unwrap(),
    };

    world_pos.0 = wnd.cursor_position().map(|screen_pos| {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        world_pos.truncate()
    });
}

#[derive(Default)]
pub struct MapCursorPos(pub Option<TilePos>);

pub fn map_cursor_pos(
    mut map_pos: ResMut<MapCursorPos>,
    world_pos: Res<WorldCursorPos>,
    active_map: Res<ActiveMap>,
) {
    let tile_pos = world_pos.and_then(|wp| active_map.to_tile_pos(wp));

    if map_pos.0 != tile_pos {
        map_pos.0 = tile_pos;
    }
}
