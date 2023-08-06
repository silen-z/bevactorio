#![feature(let_chains)]

use std::time::Duration;

use bevy::asset::ChangeWatcher;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use build_mode::BuildMode;

use crate::belts::{input_from_belts, move_items_on_belts};
use crate::build_mode::BuildModePlugin;
use crate::buildings::mine::mine_produce;
use crate::buildings::templates::loader::BuildingTemplateLoader;
use crate::buildings::templates::{
    load_building_templates, register_building_templates, BuildingRegistry, BuildingTemplate,
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
        // asset_folder: "assets".into(),
        watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
        ..default()
    };

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(window_settings)
                .set(asset_settings)
                .set(ImagePlugin::default_nearest()),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            TilemapPlugin,
            UiPlugin,
            InputPlugin,
            GridPlugin,
            BuildModePlugin,
        ))
        .add_asset::<BuildingTemplate>()
        .add_asset_loader(BuildingTemplateLoader)
        .init_resource::<Tool>()
        .init_resource::<BuildingRegistry>()
        .init_resource::<Zoom>()
        .add_event::<MapEvent>()
        .add_systems(Startup, (startup, init_map, load_building_templates))
        .add_systems(
            Update,
            (
                register_building_templates,
                camera_movement,
                mine_produce.before(move_items_on_belts),
                move_items_on_belts,
                input_from_belts.after(move_items_on_belts),
            ),
        )
        .run();
}

fn startup(mut commands: Commands, mut next_state: ResMut<NextState<BuildMode>>) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);

    next_state.set(BuildMode::Enabled);
}
