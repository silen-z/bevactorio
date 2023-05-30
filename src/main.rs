#![feature(let_chains)]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use map::{clear_buildings, init_map, should_clear_buildings};

use crate::belts::{build_belt, input_from_belts, move_items_on_belts};
use crate::buildings::chest::build_chest;
use crate::buildings::guide::{highlight_demolition, update_build_guide};
use crate::buildings::mine::{build_mine, mine_produce};
use crate::buildings::templates::{
    load_building_templates, register_building_templates, BuildingTemplate, BuildingTemplateLoader,
    BuildingTemplates,
};
use crate::buildings::{
    build_building, construct_building, demolish_building, BuildRequestedEvent, DemolishEvent,
    SelectedTool,
};
use crate::camera::{camera_movement, MainCamera, Zoom};
use crate::input::{
    handle_keyboard_input, handle_mouse_input, map_cursor_pos, world_cursor_pos, MapCursorPos,
    WorldCursorPos,
};
use crate::map::MapEvent;
use crate::ui::UiPlugin;

mod belts;
mod buildings;
mod camera;
mod direction;
mod grid;
mod input;
mod map;
mod ui;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    InGame,
    BuildMode,
}

fn main() {
    let window_settings = WindowPlugin {
        primary_window: Some(Window {
            resolution: (1270., 720.).into(),
            title: String::from("Bevactorio"),
            ..default()
        }),
        ..default()
    };

    let asset_settings = AssetPlugin {
        watch_for_changes: true,
        ..default()
    };

    let in_game_systems = (
        world_cursor_pos,
        map_cursor_pos,
        handle_mouse_input,
        handle_keyboard_input,
        camera_movement,
        build_belt.after(handle_mouse_input),
        demolish_building.after(handle_mouse_input),
        mine_produce.before(move_items_on_belts),
        move_items_on_belts,
        input_from_belts.after(move_items_on_belts),
    );

    let build_mode = (
        construct_building,
        update_build_guide,
        clear_buildings.run_if(should_clear_buildings),
        highlight_demolition.after(update_build_guide),
        build_building.run_if(on_event::<BuildRequestedEvent>()),
        build_mine.after(build_building),
        build_chest,
    );

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window_settings)
                .set(asset_settings)
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(GilrsPlugin)
        .add_state::<AppState>()
        .add_asset::<BuildingTemplate>()
        .add_asset_loader(BuildingTemplateLoader)
        .init_resource::<SelectedTool>()
        .init_resource::<BuildingTemplates>()
        .init_resource::<WorldCursorPos>()
        .init_resource::<MapCursorPos>()
        .init_resource::<Zoom>()
        .add_event::<BuildRequestedEvent>()
        .add_event::<DemolishEvent>()
        .add_event::<MapEvent>()
        .add_startup_system(startup)
        .add_startup_system(load_building_templates)
        .add_system(register_building_templates)
        .add_startup_system(init_map)
        .add_systems(in_game_systems)
        .add_systems(build_mode.in_set(OnUpdate(AppState::BuildMode)))
        .run();
}

fn startup(mut commands: Commands, mut next_state: ResMut<NextState<AppState>>) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);

    next_state.set(AppState::BuildMode);
}
