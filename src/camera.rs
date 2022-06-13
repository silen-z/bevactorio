use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

// A simple camera system for moving and zooming the camera.
pub fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
) {
    let (mut transform, mut ortho) = query.single_mut();

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::A) {
        direction -= Vec3::new(1.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::D) {
        direction += Vec3::new(1.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::W) {
        direction += Vec3::new(0.0, 1.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::S) {
        direction -= Vec3::new(0.0, 1.0, 0.0);
    }

    for e in scroll_evr.iter() {
        match e.y {
            y if y < 0. => {
                ortho.scale *= 2.;
            }
            y if y > 0. => {
                ortho.scale *= 0.5;
            }
            _ => {}
        }
    }

    if ortho.scale < 0.25 {
        ortho.scale = 0.25;
    }

    let z = transform.translation.z;
    transform.translation += time.delta_seconds() * direction * 500.;
    // Important! We need to restore the Z values when moving the camera around.
    // Bevy has a specific camera setup and this can mess with how our layers are shown.
    transform.translation.z = z;
}
