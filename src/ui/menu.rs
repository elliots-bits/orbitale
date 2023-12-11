use bevy::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::AppState;

#[derive(Component)]
struct Menu;

#[derive(Resource, Default, Clone, Copy)]
pub struct GameSettings {
    pub difficulty: Difficulty,
    pub entities_quantity: EntitiesQuantity,
    pub time_of_death: f32,
}

#[derive(Component, Default, EnumIter, Clone, Copy, PartialEq)]
pub enum Difficulty {
    GodMode,
    Easy,
    #[default]
    Normal,
    Hard,
    Impossible,
}

impl Difficulty {
    fn as_str(&self) -> &'static str {
        match self {
            Difficulty::GodMode => "God Mode",
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Impossible => "Impossible",
        }
    }
}

#[derive(Component, Default, EnumIter, Clone, Copy, PartialEq)]
pub enum EntitiesQuantity {
    Some,
    #[default]
    ALot,
    TooMuch,
}

impl EntitiesQuantity {
    fn as_str(&self) -> &'static str {
        match self {
            EntitiesQuantity::Some => "    Some", // what ? weird spaces ? yes I'm a professional why do you ask ?
            EntitiesQuantity::ALot => "a LOT !",
            EntitiesQuantity::TooMuch => "Too Much",
        }
    }
}

const PRIMARY_COLOR: Color = Color::rgb(0.95, 0.95, 0.95);
const SECONDARY_COLOR: Color = Color::rgb(0.30, 0.30, 0.30);

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct CreditsButton;

pub fn setup(app: &mut App) {
    app.insert_resource(GameSettings::default());

    app.add_systems(OnEnter(AppState::Menu), setup_menu);
    app.add_systems(OnExit(AppState::Menu), cleanup_menu);
    app.add_systems(
        Update,
        (update_menu, play_on_press_space).run_if(in_state(AppState::Menu)),
    );
}

#[derive(Component)]
pub struct MenuCameraMarker;

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setup Menu !");

    commands.spawn((Camera2dBundle::default(), MenuCameraMarker));

    let menu: Entity = commands
        .spawn((
            NodeBundle {
                style: Style {
                    // center button
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            UiImage::new(asset_server.load("nasa_milky_way.png")),
            Menu {},
        ))
        .id();

    // let new_game_title = commands
    //     .spawn((TextBundle::from_section(
    //         "New Game",
    //         TextStyle {
    //             font_size: 80.0,
    //             font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
    //             color: PRIMARY_COLOR,
    //             ..default()
    //         },
    //     )
    //     .with_style(Style {
    //         margin: UiRect {
    //             bottom: Val::Px(20.),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     }),))
    //     .id();

    let game_title = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(576.0),
                    height: Val::Px(172.0),
                    margin: UiRect {
                        // top: Val::Percent(5.),
                        bottom: Val::Percent(2.),
                        ..Default::default()
                    },
                    ..default()
                },
                // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                background_color: Color::WHITE.into(),
                ..default()
            },
            UiImage::new(asset_server.load("orbitale_title.png")),
        ))
        .id();

    commands.entity(menu).add_child(game_title);

    let difficulty_title = commands
        .spawn(TextBundle::from_section(
            "Difficulty",
            TextStyle {
                font_size: 40.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
            },
        ))
        .id();

    commands.entity(menu).add_child(difficulty_title);

    let difficulty_menu = commands
        .spawn((NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect {
                    bottom: Val::Px(30.),
                    ..Default::default()
                },
                ..default()
            },
            ..default()
        },))
        .id();

    for difficulty in Difficulty::iter() {
        let difficulty_button = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        margin: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(10.),
                            ..Default::default()
                        },
                        padding: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(10.),
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                        },
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                },
                difficulty,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    difficulty.as_str(),
                    TextStyle {
                        font_size: 40.0,
                        font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                        color: PRIMARY_COLOR,
                        ..default()
                    },
                ));
            })
            .id();

        commands
            .entity(difficulty_menu)
            .add_child(difficulty_button);
    }

    commands.entity(menu).add_child(difficulty_menu);

    let entities_quantity_title = commands
        .spawn(TextBundle::from_section(
            "Entities Quantity",
            TextStyle {
                font_size: 40.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
                ..default()
            },
        ))
        .id();

    commands.entity(menu).add_child(entities_quantity_title);

    let entities_quantity_menu = commands
        .spawn((NodeBundle {
            style: Style {
                // center button
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect {
                    bottom: Val::Px(30.),
                    ..Default::default()
                },
                ..default()
            },
            ..default()
        },))
        .id();

    for entities_quantity in EntitiesQuantity::iter() {
        let entities_quantity_button = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        margin: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(10.),
                            ..Default::default()
                        },
                        padding: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(10.),
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                        },
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                },
                entities_quantity,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    entities_quantity.as_str(),
                    TextStyle {
                        font_size: 40.0,
                        font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                        color: PRIMARY_COLOR,
                        ..default()
                    },
                ));
            })
            .id();

        commands
            .entity(entities_quantity_menu)
            .add_child(entities_quantity_button);
    }

    commands.entity(menu).add_child(entities_quantity_menu);

    let play_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        left: Val::Px(20.),
                        right: Val::Px(20.),
                        top: Val::Px(0.0),
                        bottom: Val::Px(7.),
                    },
                    border: UiRect {
                        left: Val::Px(2.),
                        right: Val::Px(2.),
                        top: Val::Px(2.),
                        bottom: Val::Px(2.),
                    },
                    top: Val::Px(5.0),
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(0.),
                        bottom: Val::Px(60.),
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: PRIMARY_COLOR.into(),
                ..default()
            },
            PlayButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Play [ ]",
                TextStyle {
                    font_size: 40.0,
                    color: PRIMARY_COLOR,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    ..default()
                },
            ));
        })
        .id();
    commands.entity(menu).add_child(play_button);

    let artists = commands
        .spawn(TextBundle::from_section(
            "Created by Elliot Bitsch & Eliott Gaboreau",
            TextStyle {
                font_size: 24.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
            },
        ))
        .id();
    commands.entity(menu).add_child(artists);

    let credits_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    padding: UiRect {
                        left: Val::Px(20.),
                        right: Val::Px(20.),
                        top: Val::Px(0.0),
                        bottom: Val::Px(7.),
                    },
                    border: UiRect {
                        left: Val::Px(2.),
                        right: Val::Px(2.),
                        top: Val::Px(2.),
                        bottom: Val::Px(2.),
                    },
                    top: Val::Px(5.0),
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(16.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: PRIMARY_COLOR.into(),
                ..default()
            },
            CreditsButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Credits",
                TextStyle {
                    font_size: 30.0,
                    color: PRIMARY_COLOR,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                },
            ));
        })
        .id();
    commands.entity(menu).add_child(credits_button);
}

fn update_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut play_button_interraction: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<PlayButton>,
            Without<CreditsButton>,
        ),
    >,
    mut credits_button_interraction: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<CreditsButton>,
            Without<PlayButton>,
        ),
    >,
    mut difficulty_button_interraction: Query<
        (&Interaction, &Difficulty),
        (
            Changed<Interaction>,
            With<Button>,
            Without<EntitiesQuantity>,
        ),
    >,
    mut difficulty_button: Query<(&Children, &Difficulty), Without<EntitiesQuantity>>,

    mut entities_quantity_button_interraction: Query<
        (&Interaction, &EntitiesQuantity),
        (Changed<Interaction>, With<Button>, Without<Difficulty>),
    >,
    mut entities_quantity_button: Query<(&Children, &EntitiesQuantity), Without<Difficulty>>,
    mut settings: ResMut<GameSettings>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, difficulty) in &mut difficulty_button_interraction {
        match *interaction {
            Interaction::Pressed => {
                settings.difficulty = *difficulty;
            }
            _ => {}
        }
    }
    for (children, difficulty) in &mut difficulty_button {
        let mut text = text_query.get_mut(children[0]).unwrap();

        text.sections[0].style.color = match *difficulty == settings.difficulty {
            true => PRIMARY_COLOR,
            false => SECONDARY_COLOR,
        }
    }

    for (interaction, entities_quantity) in &mut entities_quantity_button_interraction {
        match *interaction {
            Interaction::Pressed => {
                settings.entities_quantity = *entities_quantity;
            }
            _ => {}
        }
    }
    for (children, entities_quantity) in &mut entities_quantity_button {
        let mut text = text_query.get_mut(children[0]).unwrap();

        text.sections[0].style.color = match *entities_quantity == settings.entities_quantity {
            true => PRIMARY_COLOR,
            false => SECONDARY_COLOR,
        }
    }
    for (interaction, children, mut background_color) in &mut play_button_interraction {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Game);
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::BLACK.into();
                background_color.0 = PRIMARY_COLOR.into();
            }
            Interaction::None => {
                text.sections[0].style.color = PRIMARY_COLOR.into();
                background_color.0 = Color::NONE.into();
            }
        }
    }
    for (interaction, children, mut background_color) in &mut credits_button_interraction {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Credits);
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::BLACK.into();
                background_color.0 = PRIMARY_COLOR.into();
            }
            Interaction::None => {
                text.sections[0].style.color = PRIMARY_COLOR.into();
                background_color.0 = Color::NONE.into();
            }
        }
    }
}

fn play_on_press_space(mut next_state: ResMut<NextState<AppState>>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::Space) {
        next_state.set(AppState::Game);
    }
}

fn cleanup_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<Menu>>,
    camera_query: Query<Entity, With<MenuCameraMarker>>,
) {
    debug!("Cleanup menu");
    commands.entity(menu_query.single()).despawn_recursive();
    commands.entity(camera_query.single()).despawn_recursive();
}
