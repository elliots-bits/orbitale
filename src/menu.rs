use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use bevy_rapier2d::{
    dynamics::Velocity,
    geometry::{Collider, ColliderMassProperties},
};
use bevy_vector_shapes::{
    painter::ShapePainter,
    shapes::{DiscPainter, LinePainter, RectPainter, ThicknessType},
};
use colorgrad::CustomGradient;

use crate::{
    alien_ship::AlienShipMarker,
    camera::{GameCameraMarker, UI_LAYER},
    celestial_body::CelestialBodyMarker,
    gravity::plan_course,
    healthpoints::HealthPoints,
    player::PlayerMarker,
    AppState,
};

#[derive(Component)]
struct Menu;

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

    commands
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
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
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
                })
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
        });
}

fn update_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::Game);
            }
            _ => {} // Interaction::Hovered => {
                    //     *color = HOVERED_BUTTON.into();
                    // }
                    // Interaction::None => {
                    //     *color = NORMAL_BUTTON.into();
                    // }
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
