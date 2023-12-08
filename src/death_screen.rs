use std::{default, fmt::format};

use bevy::{
    asset::meta::Settings,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    utils::Instant,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    menu::GameSettings,
    score::{compute_score, score_multiplier, Score},
    AppState,
};

const PRIMARY_COLOR: Color = Color::rgb(0.95, 0.95, 0.95);
const SECONDARY_COLOR: Color = Color::rgb(0.30, 0.30, 0.30);

#[derive(Component)]
pub struct MenuButton;

#[derive(Component)]
pub struct ReplayButton;

#[derive(Component)]
pub struct DeathScreen;

pub fn setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::DeathScreen), setup_death_screen);
    app.add_systems(OnExit(AppState::DeathScreen), cleanup_death_screen);
    app.add_systems(
        Update,
        (update_death_screen, play_on_press_space).run_if(in_state(AppState::DeathScreen)),
    );
}

fn setup_death_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
    settings: Res<GameSettings>,
) {
    commands.spawn(Camera2dBundle::default());

    let death_screen = commands
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
            DeathScreen {},
        ))
        .id();

    let game_over_title = commands
        .spawn((TextBundle::from_section(
            " Game Over !",
            TextStyle {
                font_size: 100.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
                ..default()
            },
        )
        .with_style(Style {
            margin: UiRect {
                bottom: Val::Px(20.),
                ..Default::default()
            },
            ..Default::default()
        }),))
        .id();

    commands.entity(death_screen).add_child(game_over_title);

    let enemies_killed = commands
        .spawn(TextBundle::from_section(
            format!("Enemies killed : {}", score.enemies_killed),
            TextStyle {
                font_size: 30.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
                ..default()
            },
        ))
        .id();

    let game_duration = Instant::now().duration_since(score.time_game_start);

    let game_duration = commands
        .spawn(TextBundle::from_section(
            format!(
                "Game duration : {:02}:{:02}",
                game_duration.as_secs() / 60,
                game_duration.as_secs() % 60
            ),
            TextStyle {
                font_size: 30.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
                ..default()
            },
        ))
        .id();

    let score_multiplier = commands
        .spawn(TextBundle::from_section(
            format!("Difficulty multiplier : x{}", score_multiplier(&settings)),
            TextStyle {
                font_size: 30.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
                ..default()
            },
        ))
        .id();

    let final_score = commands
        .spawn(TextBundle::from_section(
            format!("Final Score : {}", compute_score(&score, &settings)),
            TextStyle {
                font_size: 60.0,
                font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                color: PRIMARY_COLOR,
                ..default()
            },
        ))
        .id();

    commands.entity(death_screen).add_child(enemies_killed);
    commands.entity(death_screen).add_child(game_duration);
    commands.entity(death_screen).add_child(score_multiplier);
    commands.entity(death_screen).add_child(final_score);

    let buttons_container = commands
        .spawn((NodeBundle {
            style: Style {
                top: Val::Px(30.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },))
        .id();

    let menu_button = commands
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
                        top: Val::Px(0.),
                        bottom: Val::Px(7.),
                    },
                    margin: UiRect {
                        left: Val::Px(10.),
                        right: Val::Px(10.),
                        ..Default::default()
                    },
                    border: UiRect {
                        left: Val::Px(2.),
                        right: Val::Px(2.),
                        top: Val::Px(2.),
                        bottom: Val::Px(2.),
                    },
                    top: Val::Px(5.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: PRIMARY_COLOR.into(),
                ..default()
            },
            MenuButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Menu",
                TextStyle {
                    font_size: 40.0,
                    color: PRIMARY_COLOR,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    ..default()
                },
            ));
        })
        .id();

    let replay_button = commands
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
                        top: Val::Px(0.),
                        bottom: Val::Px(7.),
                    },
                    margin: UiRect {
                        left: Val::Px(10.),
                        right: Val::Px(10.),
                        ..Default::default()
                    },
                    border: UiRect {
                        left: Val::Px(2.),
                        right: Val::Px(2.),
                        top: Val::Px(2.),
                        bottom: Val::Px(2.),
                    },
                    top: Val::Px(5.0),
                    ..default()
                },
                background_color: Color::NONE.into(),
                border_color: PRIMARY_COLOR.into(),
                ..default()
            },
            ReplayButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Replay [ ]",
                TextStyle {
                    font_size: 40.0,
                    color: PRIMARY_COLOR,
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    ..default()
                },
            ));
        })
        .id();

    commands.entity(buttons_container).add_child(menu_button);
    commands.entity(buttons_container).add_child(replay_button);

    commands.entity(death_screen).add_child(buttons_container);
}

fn update_death_screen(
    mut next_state: ResMut<NextState<AppState>>,
    mut replay_button_interraction: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<ReplayButton>,
            Without<MenuButton>,
        ),
    >,
    mut menu_button_interraction: Query<
        (&Interaction, &Children, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<MenuButton>,
            Without<ReplayButton>,
        ),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children, mut background_color) in &mut replay_button_interraction {
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

    for (interaction, children, mut background_color) in &mut menu_button_interraction {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Menu);
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

fn cleanup_death_screen(
    mut commands: Commands,
    death_screen_query: Query<Entity, With<DeathScreen>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    commands
        .entity(death_screen_query.single())
        .despawn_recursive();
    commands.entity(camera_query.single()).despawn_recursive();
}
