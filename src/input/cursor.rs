use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_ecs_tilemap::prelude::*;

use crate::camera::MainCamera;
use crate::grid::GridLayer;
use crate::map::to_tile_pos;

#[derive(Resource, Default, Debug)]
pub struct GameCursor {
    pub world_pos: Option<Vec2>,
    pub tile_pos: Option<TilePos>,
}

pub fn update_world_cursor(
    windows: Query<(Entity, &Window, Option<&PrimaryWindow>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut cursor: ResMut<GameCursor>,
) {
    let (camera, camera_transform) = camera_query.single();

    let window = match camera.target {
        RenderTarget::Window(WindowRef::Primary) => {
            let Some((_,window,_)) = windows.iter().find(|(_,_,primary)| primary.is_some()) else {
                return;
            };
            window
        }
        RenderTarget::Window(WindowRef::Entity(e)) => {
            let Some((_,window,_)) = windows.iter().find(|(entity,_,_)| *entity == e) else {
                return;
            };
            window
        }
        _ => return,
    };

    let Some(screen_pos) = window.cursor_position() else {
        return;
    };

    if let Some(ray) = camera.viewport_to_world(camera_transform, screen_pos) {
        cursor.world_pos = Some(ray.origin.truncate());
    }
}

pub fn update_map_cursor(
    mut cursor: ResMut<GameCursor>,
    grid_layer_query: Query<(&TilemapSize, &TilemapTileSize, &Transform), With<GridLayer>>,
) {
    let Some(world_pos) = cursor.world_pos else {
        return;
    };

    let (map_size, tile_size, map_transform) = grid_layer_query.single();
    let tile_pos = to_tile_pos(world_pos, tile_size, map_size, map_transform);

    if tile_pos != cursor.tile_pos {
        cursor.tile_pos = tile_pos;
    }
}
