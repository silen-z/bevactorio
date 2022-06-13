use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::belts::{build_belt, move_items_on_belts};
use crate::buildings::{
    demolish_building, update_build_guide, BuildEvent, DemolishEvent, SelectedBuilding,
};
use crate::camera::{camera_movement, MainCamera};
use crate::input::{
    handle_keyboard_input, handle_mouse_input, handle_wheel_input, map_cursor_pos,
    world_cursor_pos, MapCursorPos, WorldCursorPos,
};
use crate::map::ActiveMap;
use crate::mine::{build_mine, mine_produce};

mod belts;
mod buildings;
mod camera;
mod input;
mod map;
mod mine;

fn startup(mut commands: Commands, mut app_state: ResMut<State<AppState>>) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);

    let _ = app_state.push(AppState::BuildMode);
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    InGame,
    BuildMode,
}

fn main() {
    let window_settings = WindowDescriptor {
        width: 1270.0,
        height: 720.0,
        title: String::from("Bevactorio"),
        ..Default::default()
    };

    let in_game_systems = SystemSet::on_in_stack_update(AppState::InGame)
        .with_system(world_cursor_pos)
        .with_system(map_cursor_pos)
        .with_system(handle_mouse_input)
        .with_system(handle_wheel_input)
        .with_system(handle_keyboard_input)
        .with_system(camera_movement)
        .with_system(build_belt.after(handle_mouse_input))
        .with_system(build_mine.after(handle_mouse_input))
        .with_system(demolish_building.after(handle_mouse_input))
        .with_system(mine_produce)
        .with_system(move_items_on_belts.after(mine_produce))
        .with_system(set_texture_filters_to_nearest);

    let build_mode = SystemSet::on_update(AppState::BuildMode).with_system(update_build_guide);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_state(AppState::InGame)
        .init_resource::<ActiveMap>()
        .init_resource::<SelectedBuilding>()
        .init_resource::<WorldCursorPos>()
        .init_resource::<MapCursorPos>()
        .insert_resource(window_settings)
        .add_event::<BuildEvent>()
        .add_event::<DemolishEvent>()
        .add_startup_system(startup)
        .add_system_set(in_game_systems)
        .add_system_set(build_mode)
        .run();
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    use bevy::render::render_resource::TextureUsages;
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST;
            }
        }
    }
}
