#![feature(let_chains)]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use buildings::{update_build_guide, highlight_demolition};
use map::{init_map, toggle_grid, clear_buildings};

use crate::belts::{build_belt, input_from_belts, move_items_on_belts};
use crate::buildings::chest::build_chest;
use crate::buildings::mine::{build_mine, mine_produce};
use crate::buildings::templates::BuildingTemplates;
use crate::buildings::{
    build_building, demolish_building, BuildRequestedEvent, BuildingBuiltEvent, DemolishEvent,
    SelectedTool,
};
use crate::camera::{camera_movement, MainCamera, Zoom};
use crate::input::{
    handle_keyboard_input, handle_mouse_input, map_cursor_pos, world_cursor_pos, MapCursorPos,
    WorldCursorPos,
};
use crate::map::{Grid, MapEvent};
use crate::ui::{
    handle_select_tool, highlight_selected_tool, init_ui, track_ui_interaction, MapInteraction,
};

mod belts;
mod buildings;
mod camera;
mod input;
mod map;
mod ui;

fn startup(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);

    next_state.set(AppState::BuildMode);
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    InGame,
    BuildMode,
}

fn main() {
    let window_settings = Window {
        resolution: (1270., 720.).into(),
        title: String::from("Bevactorio"),
        ..default()
    };

    let in_game_systems = (
        world_cursor_pos,
        map_cursor_pos,
        track_ui_interaction,
        handle_mouse_input,
        handle_keyboard_input,
        camera_movement,
        toggle_grid.after(handle_keyboard_input),
        build_belt.after(handle_mouse_input),
        demolish_building.after(handle_mouse_input),
        mine_produce.before(move_items_on_belts),
        move_items_on_belts,
        input_from_belts.after(move_items_on_belts),
    );

    let build_mode = (
        update_build_guide,
        handle_select_tool,
        clear_buildings,
        highlight_selected_tool,
        highlight_demolition.after(update_build_guide),
        build_building,
        build_mine.after(build_building),
        build_chest,
    );

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(window_settings),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_state::<AppState>()
        .init_resource::<SelectedTool>()
        .init_resource::<BuildingTemplates>()
        .init_resource::<WorldCursorPos>()
        .init_resource::<MapCursorPos>()
        .init_resource::<Grid>()
        .init_resource::<MapInteraction>()
        .init_resource::<Zoom>()
        .add_event::<BuildRequestedEvent>()
        .add_event::<BuildingBuiltEvent>()
        .add_event::<DemolishEvent>()
        .add_event::<MapEvent>()
        .add_startup_system(startup)
        .add_startup_system(init_ui)
        .add_startup_system(init_map)
        .add_systems(in_game_systems)
        .add_systems(build_mode.in_set(OnUpdate(AppState::BuildMode)))
        .run();
}
