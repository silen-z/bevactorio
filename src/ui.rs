use bevy::prelude::*;

use crate::buildings::{BuildTool, BuildingType, Tool};

#[derive(Component, Clone)]
pub struct SelectToolAction(Tool, fn(&Tool, &Tool) -> bool);

impl SelectToolAction {
    fn is_same(&self, other: &Tool) -> bool {
        self.1(&self.0, other)
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapInteraction>()
            .add_systems(Startup, init_ui)
            .add_systems(
                Update,
                (
                    handle_select_tool,
                    highlight_selected_tool,
                    track_ui_interaction,
                ),
            );
    }
}

pub fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut building_menu = commands.spawn(NodeBundle {
        background_color: Color::NONE.into(),
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            position_type: PositionType::Absolute,

            top: Val::Px(16.),
            left: Val::Px(16.),

            ..default()
        },
        ..default()
    });

    let font = asset_server.load("AsepriteFont.ttf");

    let button_builder = |parent: &mut ChildBuilder, text, action| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        left: Val::Px(16.),
                        right: Val::Px(16.),
                        top: Val::Px(8.),
                        bottom: Val::Px(8.),
                    },
                    margin: UiRect {
                        bottom: Val::Px(16.),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            })
            .insert(action)
            .with_children(|button| {
                button.spawn(TextBundle { text, ..default() });
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
            SelectToolAction(
                Tool::Build(BuildTool {
                    building: BuildingType::Belt,
                    direction: default(),
                }),
                PartialEq::eq,
            ),
        );

        button_builder(
            menu,
            button_text("BUILD ".to_string(), 'M', "INE".to_string()),
            SelectToolAction(
                Tool::Build(BuildTool {
                    building: BuildingType::Mine,
                    direction: default(),
                }),
                is_same_building_tool,
            ),
        );

        button_builder(
            menu,
            button_text("PLACE ".to_string(), 'C', "HEST".to_string()),
            SelectToolAction(
                Tool::Build(BuildTool {
                    building: BuildingType::Chest,
                    direction: default(),
                }),
                PartialEq::eq,
            ),
        );

        button_builder(
            menu,
            button_text("".to_string(), 'D', "EMOLISH".to_string()),
            SelectToolAction(Tool::Buldozer, PartialEq::eq),
        );
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(16.),
                bottom: Val::Px(16.),
                padding: UiRect::all(Val::Px(16.)),
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        })
        .with_children(|help_box| {
            help_box.spawn(TextBundle {
                text: Text::from_section(
                    HELP_TEXT.to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: Color::DARK_GRAY,
                        ..default()
                    },
                ),
                ..default()
            });
        });
}

pub fn handle_select_tool(
    actions: Query<(&SelectToolAction, &Interaction), Changed<Interaction>>,
    mut selected_tool: ResMut<Tool>,
) {
    if let Some((action, _)) = actions
        .iter()
        .find(|(_, interaction)| matches!(interaction, Interaction::Pressed))
    {
        *selected_tool = action.0.clone();
    }
}

pub fn highlight_selected_tool(
    selected_tool: Res<Tool>,
    mut elements: Query<(Entity, &mut BackgroundColor, &SelectToolAction)>,
    mut highlighted_nodes: Local<Vec<(Entity, BackgroundColor)>>,
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
            if action.is_same(&selected_tool) {
                highlighted_nodes.push((entity, *color));
                *color = Color::GRAY.into();
            }
        }
    }
}

#[derive(Resource, Default)]
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
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));

    map_interaction.0 = !is_interacting_with_ui;
}

fn is_same_building_tool(a: &Tool, b: &Tool) -> bool {
    match (a, b) {
        (Tool::Build(tool_a), Tool::Build(tool_b)) => tool_a.building == tool_b.building,
        _ => false,
    }
}

const HELP_TEXT: &str = r#"
arrows - move camera
scroll - camera zoom
G - toggle grid
C - clear all buildings

hightlighted shortcuts in build menu
"#;
