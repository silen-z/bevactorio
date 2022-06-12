use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

use crate::MainCamera;

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
