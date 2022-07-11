#![feature(let_else)]
#![feature(let_chains)]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::belts::{build_belt, input_from_belts, move_items_on_belts};
use crate::buildings::chest::build_chest;
use crate::buildings::mine::{build_mine, mine_produce};
use crate::buildings::templates::{
    load_building_templates, register_building_templates, BuildingTemplates, BuildingTilemap,
    BuildingTilemapLoader,
};
use crate::buildings::{
    build_building, demolish_building, highlight_demolition, update_build_guide,
    BuildRequestedEvent, BuildingBuiltEvent, DemolishEvent, SelectedTool,
};
use crate::camera::{camera_movement, MainCamera, Zoom};
use crate::input::{
    handle_keyboard_input, handle_mouse_input, map_cursor_pos, world_cursor_pos, MapCursorPos,
    WorldCursorPos,
};
use crate::map::{clear_buildings, toggle_grid, ActiveMap, GridState, MapEvent};
use crate::ui::{
    handle_select_tool, highlight_selected_tool, init_ui, track_ui_interaction, MapInteraction,
};

mod belts;
mod buildings;
mod camera;
mod direction;
mod input;
mod map;
mod ui;

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
        ..default()
    };

    let in_game_systems = SystemSet::on_in_stack_update(AppState::InGame)
        .with_system(world_cursor_pos)
        .with_system(map_cursor_pos)
        .with_system(track_ui_interaction)
        .with_system(handle_mouse_input)
        .with_system(handle_keyboard_input)
        .with_system(camera_movement)
        .with_system(toggle_grid.after(handle_keyboard_input))
        .with_system(build_belt.after(handle_mouse_input))
        .with_system(demolish_building.after(handle_mouse_input))
        .with_system(mine_produce.before(move_items_on_belts))
        .with_system(move_items_on_belts)
        .with_system(set_texture_filters_to_nearest)
        .with_system(input_from_belts.after(move_items_on_belts));

    let build_mode = SystemSet::on_update(AppState::BuildMode)
        .with_system(update_build_guide)
        .with_system(handle_select_tool)
        .with_system(clear_buildings)
        .with_system(highlight_selected_tool)
        .with_system(highlight_demolition.after(update_build_guide))
        .with_system(build_building)
        .with_system(build_mine.after(build_building))
        .with_system(build_chest);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_state(AppState::InGame)
        .add_asset::<BuildingTilemap>()
        .add_asset_loader(BuildingTilemapLoader)
        .init_resource::<ActiveMap>()
        .init_resource::<SelectedTool>()
        .init_resource::<BuildingTemplates>()
        .init_resource::<WorldCursorPos>()
        .init_resource::<MapCursorPos>()
        .init_resource::<GridState>()
        .init_resource::<MapInteraction>()
        .init_resource::<Zoom>()
        .insert_resource(window_settings)
        .add_event::<BuildRequestedEvent>()
        .add_event::<BuildingBuiltEvent>()
        .add_event::<DemolishEvent>()
        .add_event::<MapEvent>()
        .add_startup_system(startup)
        .add_startup_system(init_ui)
        .add_startup_system(load_building_templates)
        .add_system(register_building_templates)
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
