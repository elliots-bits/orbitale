use std::default;

use bevy::{asset::meta::Settings, prelude::*};
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

#[derive(Component, Default, EnumIter)]
pub enum Difficulty {
    Unkillable,
    Easy,
    #[default]
    Normal,
    Hard,
    Impossible,
}

impl Difficulty {
    fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Unkillable => "Unkillable",
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
    app.add_systems(OnEnter(AppState::DeathScreen), on_death);

    app.add_systems(OnEnter(AppState::Menu), setup_menu);
    app.add_systems(OnExit(AppState::Menu), cleanup_menu);
    app.add_systems(Update, update_menu.run_if(in_state(AppState::Menu)));
}

fn on_death(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Menu);
}

fn setup_menu(mut commands: Commands) {
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
                    ..default()
                },
                ..default()
            },
            Menu {},
        ))
        .id();

    let difficulty_menu = commands
        .spawn((NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
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
                difficulty,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    difficulty.as_str(),
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ));
            })
            .id();
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
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });

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
    mut settings: ResMut<GameSettings>,
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
                // Change old difficulty button's color

                settings.difficulty = *difficulty;
                // Change new difficulty button's color
            }
            Interaction::Hovered => {}
            _ => {}
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
