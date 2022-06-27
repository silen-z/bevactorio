use bevy::prelude::*;

use crate::buildings::{BuildingType, SelectedTool};

#[derive(Component, Clone)]
pub struct SelectToolAction(SelectedTool);

pub fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle { ..default() });
    let mut building_menu = commands.spawn_bundle(NodeBundle {
        color: Color::NONE.into(),
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
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

    let font = asset_server.load("AsepriteFont.ttf");

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
                button.spawn_bundle(TextBundle { text, ..default() });
            });
    };

    building_menu.with_children(|menu| {
        let button_text = |before, shortcut: char, after| {
            let style = TextStyle {
                font: font.clone(),
                color: Color::DARK_GRAY,
                font_size: 24.,
                ..default()
            };

            let sections = vec![
                TextSection {
                    value: before,
                    style: style.clone(),
                },
                TextSection {
                    value: shortcut.into(),
                    style: TextStyle {
                        color: Color::rgb_u8(179, 24, 0),
                        ..style.clone()
                    },
                },
                TextSection {
                    value: after,
                    style,
                },
            ];

            Text {
                sections,
                ..default()
            }
        };

        button_builder(
            menu,
            button_text("LAY ".to_string(), 'B', "ELTS".to_string()),
            SelectToolAction(SelectedTool::Build {
                building: BuildingType::Belt,
                direction: default()
            }),
        );

        button_builder(
            menu,
            button_text("BUILD ".to_string(), 'M', "INE".to_string()),
            SelectToolAction(SelectedTool::Build {
                building: BuildingType::Mine,
                direction: default()
            }),
        );

        button_builder(
            menu,
            button_text("PLACE ".to_string(), 'C', "HEST".to_string()),
            SelectToolAction(SelectedTool::Build {
                building: BuildingType::Chest,
                direction: default()
            }),
        );

        button_builder(
            menu,
            button_text("".to_string(), 'D', "EMOLISH".to_string()),
            SelectToolAction(SelectedTool::Buldozer),
        );
    });

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(16.),
                    bottom: Val::Px(16.),
                    ..default()
                },
                padding: Rect::all(Val::Px(16.)),
                ..default()
            },
            color: Color::WHITE.into(),
            ..default()
        })
        .with_children(|help_box| {
            help_box.spawn_bundle(TextBundle {
                text: Text::with_section(
                    HELP_TEXT.to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: Color::DARK_GRAY,
                        ..default()
                    },
                    TextAlignment::default(),
                ),
                ..default()
            });
        });
}

pub fn handle_select_tool(
    actions: Query<(&SelectToolAction, &Interaction), Changed<Interaction>>,
    mut selected_building: ResMut<SelectedTool>,
) {
    if let Some((action, _)) = actions
        .iter()
        .find(|(_, interaction)| matches!(interaction, Interaction::Clicked))
    {
        *selected_building = action.0.clone();
    }
}

pub fn highlight_selected_tool(
    selected_tool: Res<SelectedTool>,
    mut elements: Query<(Entity, &mut UiColor, &SelectToolAction)>,
    mut highlighted_nodes: Local<Vec<(Entity, UiColor)>>,
) {
    if selected_tool.is_changed() {
        // clear highlights
        for (entity, previous_color) in highlighted_nodes.drain(..) {
            if let Ok((_, mut color, _)) = elements.get_mut(entity) {
                *color = previous_color;
            }
        }

        // apply new highlights
        for (entity, mut color, action) in elements.iter_mut() {
            if action.0 == *selected_tool {
                highlighted_nodes.push((entity, *color));
                *color = Color::GRAY.into();
            }
        }
    }
}

#[derive(Default)]
pub struct MapInteraction(bool);

impl MapInteraction {
    pub fn is_allowed(&self) -> bool {
        self.0
    }
}

pub fn track_ui_interaction(
    ui_components: Query<&Interaction>,
    mut map_interaction: ResMut<MapInteraction>,
) {
    let is_interacting_with_ui = ui_components
        .iter()
        .any(|i| matches!(i, Interaction::Clicked | Interaction::Hovered));

    map_interaction.0 = !is_interacting_with_ui;
}

const HELP_TEXT: &str = r#"
arrows - move camera
scroll - camera zoom
G - toggle grid
C - clear all buildings

hightlighted shortcuts in build menu
"#;
