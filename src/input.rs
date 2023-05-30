use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

pub use self::cursor::GameCursor;
use crate::buildings::{BuildRequestedEvent, BuildTool, BuildingType, DemolishEvent, Tool};
use crate::map::MapEvent;
use crate::ui::MapInteraction;

pub mod cursor;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameCursor>()
            .add_system(handle_mouse_input)
            .add_system(handle_keyboard_input)
            .add_systems((cursor::update_world_cursor, cursor::update_map_cursor).chain());
    }
}

pub fn handle_mouse_input(
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<GameCursor>,
    mut build_events: EventWriter<BuildRequestedEvent>,
    mut demolish_events: EventWriter<DemolishEvent>,
    map_interaction: Res<MapInteraction>,
    selected_tool: Res<Tool>,
) {
    let Some(tile_pos) = cursor_pos.tile_pos else {
        return;
    };

    if mouse.pressed(MouseButton::Left) && map_interaction.is_allowed() {
        match &*selected_tool {
            Tool::Build(build_tool) => {
                build_events.send(build_tool.request_at(tile_pos));
            }

            Tool::Buldozer => {
                demolish_events.send(DemolishEvent { tile_pos });
            }

            _ => {}
        }
    }
}

pub fn handle_keyboard_input(
    mut key_events: EventReader<KeyboardInput>,
    mut map_events: EventWriter<MapEvent>,
    mut selected_tool: ResMut<Tool>,
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
                *selected_tool = Tool::Build(BuildTool {
                    building: BuildingType::Mine,
                    direction: default(),
                })
            }

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::B),
                ..
            } => {
                *selected_tool = Tool::Build(BuildTool {
                    building: BuildingType::Belt,
                    direction: default(),
                })
            }

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::D),
                ..
            } => *selected_tool = Tool::Buldozer,

            KeyboardInput {
                state: ButtonState::Pressed,
                key_code: Some(KeyCode::R),
                ..
            } => selected_tool.rotate(),

            _ => {}
        }
    }
}
