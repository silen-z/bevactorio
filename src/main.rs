#![feature(let_chains)]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use build_mode::BuildMode;

use crate::belts::{input_from_belts, move_items_on_belts};
use crate::build_mode::BuildModePlugin;
use crate::buildings::mine::mine_produce;
use crate::buildings::templates::{
    load_building_templates, register_building_templates, BuildingTemplate, BuildingTemplateLoader,
    BuildingRegistry,
};
use crate::buildings::Tool;
use crate::camera::{camera_movement, MainCamera, Zoom};
use crate::grid::GridPlugin;
use crate::input::InputPlugin;
use crate::map::{init_map, MapEvent};
use crate::ui::UiPlugin;

mod belts;
mod build_mode;
mod buildings;
mod camera;
mod direction;
mod grid;
mod input;
mod map;
mod ui;

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
        camera_movement,
        mine_produce.before(move_items_on_belts),
        move_items_on_belts,
        input_from_belts.after(move_items_on_belts),
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
        .add_plugin(InputPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(BuildModePlugin)
        .add_asset::<BuildingTemplate>()
        .add_asset_loader(BuildingTemplateLoader)
        .init_resource::<Tool>()
        .init_resource::<BuildingRegistry>()
        .init_resource::<Zoom>()
        .add_event::<MapEvent>()
        .add_startup_system(startup)
        .add_startup_system(load_building_templates)
        .add_system(register_building_templates)
        .add_startup_system(init_map)
        .add_systems(in_game_systems)
        .run();
}                                  

fn startup(mut commands: Commands, mut next_state: ResMut<NextState<BuildMode>>) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);

    next_state.set(BuildMode::Enabled);
}
