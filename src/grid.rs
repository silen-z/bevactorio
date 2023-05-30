
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::filling::fill_tilemap;
use bevy_ecs_tilemap::prelude::*;

use crate::camera::Zoom;
use crate::input::handle_keyboard_input;
use crate::map::{MapEvent, GRID_SIZE, TILEMAP_SIZE, TILE_SIZE};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Grid>()
            .add_startup_system(create_grid_layer)
            .add_system(toggle_grid.after(handle_keyboard_input));
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub enum Grid {
    #[default]
    Enabled,
    Disabled,
}

impl std::ops::Not for Grid {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Grid::Enabled => Grid::Disabled,
            Grid::Disabled => Grid::Enabled,
        }
    }
}

impl From<bool> for Grid {
    fn from(value: bool) -> Self {
        match value {
            true => Grid::Enabled,
            false => Grid::Disabled,
        }
    }
}

impl From<Grid> for bool {
    fn from(value: Grid) -> Self {
        match value {
            Grid::Enabled => true,
            Grid::Disabled => false,
        }
    }
}

impl Grid {
    pub fn toggle(&mut self) {
        *self = !*self;
    }
}

#[derive(Component)]
pub struct GridLayer;

const MAX_GRID_ZOOM: f32 = 2.;

pub fn create_grid_layer(mut commands: Commands, asset_server: Res<AssetServer>) {
    let grid_texture = asset_server.load("tilesets/grid.png");

    let grid_tilemap = commands.spawn(GridLayer).id();

    let mut grid_storage = TileStorage::empty(TILEMAP_SIZE);
    fill_tilemap(
        TileTextureIndex(0),
        TILEMAP_SIZE,
        TilemapId(grid_tilemap),
        &mut commands,
        &mut grid_storage,
    );

    commands.entity(grid_tilemap).insert(TilemapBundle {
        grid_size: GRID_SIZE,
        size: TILEMAP_SIZE,
        storage: grid_storage,
        texture: TilemapTexture::Single(grid_texture),
        tile_size: TILE_SIZE,
        transform: bevy_ecs_tilemap::helpers::geometry::get_tilemap_center_transform(
            &TILEMAP_SIZE,
            &GRID_SIZE,
            &TilemapType::Square,
            3.0,
        ),
        ..default()
    });
}

pub fn toggle_grid(
    mut grid_layer: Query<&mut Visibility, With<GridLayer>>,
    mut map_events: EventReader<MapEvent>,
    mut grid_state: ResMut<Grid>,
    zoom: Res<Zoom>,
) {
    for _ in map_events
        .iter()
        .filter(|e| matches!(e, MapEvent::ToggleGrid))
    {
        grid_state.toggle();
    }

    if !grid_state.is_changed() && !zoom.is_changed() {
        return;
    }

    if let Ok(mut grid_visibility) = grid_layer.get_single_mut() {
        *grid_visibility = match matches!(*grid_state, Grid::Enabled) && zoom.0 < MAX_GRID_ZOOM {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        }
    };
}
