use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseWheel;

use crate::belts::build_belt;
use crate::buildings::{demolish_building, BuildingType, SelectedBuilding};
use crate::camera::camera_movement;
use crate::map::build_map;
use crate::mouse::{world_cursor_pos, WorldCursorPos};

mod belts;
mod buildings;
mod camera;
mod map;
mod mouse;

fn select_building(
    mut scroll_evr: EventReader<MouseWheel>,
    mut selected_building: ResMut<SelectedBuilding>,
) {
    for event in scroll_evr.iter() {
        match event.y {
            y if y < 0. => selected_building.prev(),
            y if y > 0. => selected_building.next(),
            _ => {}
        };
    }
}

fn mouse_input(
    mouse: Res<Input<MouseButton>>,
    map_pos: Res<WorldCursorPos>,
    mut build_events: EventWriter<BuildEvent>,
    mut demolish_events: EventWriter<DemolishEvent>,
    mut last_placed: ResMut<LastPlacedTile>,
    selected_building: Res<SelectedBuilding>,
) {
    if let Some(cursor_pos) = map_pos.0 {
        let tile_pos = {
            let x = cursor_pos.x / 16. + 24.;
            let y = cursor_pos.y / 16. + 24.;

            if x < 0. || y < 0. {
                return;
            }

            TilePos(x as u32, y as u32)
        };

        if mouse.pressed(MouseButton::Left) {
            build_events.send(BuildEvent {
                building_type: selected_building.get(),
                tile_pos,
            });
        }

        if mouse.pressed(MouseButton::Right) {
            demolish_events.send(DemolishEvent { tile_pos });
        }
    }

    if mouse.just_released(MouseButton::Left) {
        last_placed.0 = None;
    }
}

#[derive(Default, Deref)]
pub struct LastPlacedTile(Option<(Entity, TilePos)>);

pub struct BuildEvent {
    building_type: BuildingType,
    tile_pos: TilePos,
}

pub struct DemolishEvent {
    tile_pos: TilePos,
}

#[derive(Component)]
pub struct MainCamera;

fn startup(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Random Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_startup_system(build_map)
        .init_resource::<WorldCursorPos>()
        .add_system(world_cursor_pos)
        .add_system(select_building)
        .add_system(mouse_input)
        .init_resource::<SelectedBuilding>()
        .init_resource::<LastPlacedTile>()
        .add_event::<BuildEvent>()
        .add_event::<DemolishEvent>()
        .add_system(build_belt.after(mouse_input))
        .add_system(demolish_building.after(mouse_input))
        .add_system(camera_movement)
        .add_system(set_texture_filters_to_nearest)
        .run();
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    use bevy::render::render_resource::TextureUsages;
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}
