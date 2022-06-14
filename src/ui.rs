use bevy::prelude::*;

use crate::buildings::{BuildingType, SelectedTool};

#[derive(Component, Clone)]
pub struct SelectToolAction(SelectedTool);

pub fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle { ..default() });
    let mut building_menu = commands.spawn_bundle(NodeBundle {
        color: Color::NONE.into(),
        style: Style {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(16.),
                left: Val::Px(16.),
                ..default()
            },
            ..default()
        },
        ..default()
    });

    let button_builder = |parent: &mut ChildBuilder, text, action| {
        parent
            .spawn_bundle(ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: Rect {
                        left: Val::Px(16.),
                        right: Val::Px(16.),
                        top: Val::Px(8.),
                        bottom: Val::Px(8.),
                    },
                    margin: Rect {
                        bottom: Val::Px(16.),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            })
            .insert(action)
            .with_children(|button| {
                button.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        text,
                        TextStyle {
                            font: asset_server.load("AsepriteFont.ttf"),
                            color: Color::GRAY,
                            font_size: 24.,
                            ..default()
                        },
                        default(),
                    ),
                    ..default()
                });
            });
    };

    building_menu.with_children(|menu| {
        button_builder(
            menu,
            "LAY BELTS".to_string(),
            SelectToolAction(SelectedTool::Building(BuildingType::Belt)),
        );
        button_builder(
            menu,
            "BUILD MINE".to_string(),
            SelectToolAction(SelectedTool::Building(BuildingType::Mine)),
        );

        button_builder(
            menu,
            "DEMOLISH".to_string(),
            SelectToolAction(SelectedTool::Buldozer),
        );
    });
}

pub fn handle_select_tool(
    actions: Query<(&SelectToolAction, &Interaction)>,
    mut selected_building: ResMut<SelectedTool>,
) {
    if let Some((action, _)) = actions
        .iter()
        .find(|(_, interaction)| matches!(interaction, Interaction::Clicked))
    {
        *selected_building = action.0.clone();
    }
}
