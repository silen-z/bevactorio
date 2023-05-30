use bevy::prelude::*;

use crate::belts::build_belt;
use crate::buildings::chest::build_chest;
use crate::buildings::guide::{should_update_build_guide, update_build_guide, update_demo_guide};
use crate::buildings::mine::build_mine;
use crate::buildings::{
    build_building, construct_building, demolish_building, BuildRequestedEvent, DemolishEvent,
};
use crate::input::handle_mouse_input;
use crate::map::{clear_buildings, should_clear_buildings};

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum BuildMode {
    Enabled,
    #[default]
    Disabled,
}

pub struct BuildModePlugin;

impl Plugin for BuildModePlugin {
    fn build(&self, app: &mut App) {
        let build_mode = (
            construct_building,
            demolish_building.after(handle_mouse_input),
            clear_buildings.run_if(should_clear_buildings),
            build_building.run_if(on_event::<BuildRequestedEvent>()),
            build_belt.after(handle_mouse_input),
            build_mine.after(build_building),
            build_chest,
        );

        let build_guide_systems = (update_build_guide, update_demo_guide)
            .chain()
            .distributive_run_if(should_update_build_guide);

        app.add_state::<BuildMode>()
            .add_event::<BuildRequestedEvent>()
            .add_event::<DemolishEvent>()
            .add_systems(build_mode.in_set(OnUpdate(BuildMode::Enabled)))
            .add_systems(build_guide_systems.in_set(OnUpdate(BuildMode::Enabled)));
    }
}
