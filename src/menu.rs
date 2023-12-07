use std::default;

use bevy::{
    asset::meta::Settings,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::AppState;

#[derive(Component)]
struct Menu;

#[derive(Resource, Default)]
struct GameSettings {
    difficulty: Difficulty,
    entities_quantity: EntitiesQuantity,
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

#[derive(Component, Default, EnumIter)]
pub enum EntitiesQuantity {
    Some,
    #[default]
    ALot,
    TooMuch,
}

#[derive(Component)]
pub struct PlayButton;

pub fn setup(app: &mut App) {
    app.insert_resource(GameSettings::default());
    app.add_systems(OnEnter(AppState::DeathScreen), on_death);

    app.add_systems(OnEnter(AppState::Menu), setup_menu);
    app.add_systems(OnExit(AppState::Menu), cleanup_menu);
    app.add_systems(Update, update_menu.run_if(in_state(AppState::Menu)));
}

fn on_death(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Menu);
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setup Menu !");

    commands.spawn(Camera2dBundle::default());

    let menu = commands
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

    let difficulty_menu = commands
        .spawn((NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(30.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
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
                        color: Color::rgb(5.0, 5.0, 5.0),
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

    let play_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                ..default()
            },
            PlayButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Play",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(5.0, 5.0, 5.0),
                    ..default()
                },
            ));
        })
        .id();

    commands.entity(menu).add_child(play_button);
}

fn update_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut play_button_interraction: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>, With<PlayButton>),
    >,
    mut difficulty_button_interraction: Query<
        (&Interaction, &Difficulty),
        (Changed<Interaction>, With<Button>),
    >,
    mut difficulty_button: Query<(&mut Children, &Difficulty)>,
    mut settings: ResMut<GameSettings>,
    mut text_query: Query<&mut Text>,
) {
    for interaction in &mut play_button_interraction {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Game);
            }
            Interaction::Hovered => {}
            _ => {}
        }
    }
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
            true => Color::rgb(5.0, 5.0, 5.0),
            false => Color::rgb(0.15, 0.15, 0.15),
        }
    }
}

fn cleanup_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<Menu>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    commands.entity(menu_query.single()).despawn_recursive();
    commands.entity(camera_query.single()).despawn_recursive();
}
