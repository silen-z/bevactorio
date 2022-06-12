use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::input::ElementState;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_ecs_tilemap::prelude::*;

use crate::buildings::{BuildEvent, DemolishEvent, SelectedBuilding};
use crate::camera::MainCamera;
use crate::map::{ActiveMap, MapLayer};

pub fn handle_mouse_input(
    mouse: Res<Input<MouseButton>>,
    map_pos: Res<WorldCursorPos>,
    mut build_events: EventWriter<BuildEvent>,
    mut demolish_events: EventWriter<DemolishEvent>,
    selected_building: Res<SelectedBuilding>,
    active_map: Res<ActiveMap>,
) {
    if let Some(tile_pos) = map_pos.and_then(|cursor_pos| active_map.to_tile_pos(cursor_pos)) {
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
}

pub fn handle_wheel_input(
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

pub fn handle_keyboard_input(
    mut commands: Commands,
    mut key_events: EventReader<KeyboardInput>,
    mut layers: Query<&mut Transform>,
    mut map_query: MapQuery,
    active_map: Res<ActiveMap>,
) {
    for event in key_events.iter() {
        match event {
            KeyboardInput {
                state: ElementState::Pressed,
                key_code: Some(KeyCode::G),
                ..
            } => {
                if let Some(mut transform) = map_query
                    .get_layer(active_map.map_id, MapLayer::Grid)
                    .and_then(|(e, _)| layers.get_mut(e).ok())
                {
                    transform.translation.z = if transform.translation.z < 0. {
                        u16::from(MapLayer::Grid) as f32
                    } else {
                        -10.0
                    };
                }
            }
            KeyboardInput {
                state: ElementState::Pressed,
                key_code: Some(KeyCode::C),
                ..
            } => {
                map_query.despawn_layer_tiles(
                    &mut commands,
                    active_map.map_id,
                    MapLayer::Buildings,
                );
                if let Some((_, layer)) =
                    map_query.get_layer(active_map.map_id, MapLayer::Buildings)
                {
                    let chunks = (0..layer.settings.map_size.0)
                        .flat_map(|x| (0..layer.settings.map_size.1).map(move |y| (x, y)))
                        .flat_map(|(x, y)| layer.get_chunk(ChunkPos(x, y)))
                        .collect::<Vec<_>>();

                    for chunk in chunks {
                        map_query.notify_chunk(chunk);
                    }
                }
            }
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
