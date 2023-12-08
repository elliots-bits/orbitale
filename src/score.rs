use std::time::Duration;

use bevy::{prelude::*, transform::commands, utils::Instant};

use crate::AppState;

#[derive(Resource)]
pub struct Score {
    pub enemies_killed: u32,
    pub time_game_start: Instant,
}

#[derive(Component)]
pub struct ScoreHudText;

pub fn setup(app: &mut App) {
    app.insert_resource(Score {
        enemies_killed: 0,
        time_game_start: Instant::now(),
    });
    app.add_systems(OnEnter(AppState::Game), setup_score_hud);
    app.add_systems(OnExit(AppState::Game), cleanup_score_hud);
    app.add_systems(Update, update_score_hud.run_if(in_state(AppState::Game)));
}

fn setup_score_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    score.enemies_killed = 0;
    score.time_game_start = Instant::now();

    commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Score: 0",
                    TextStyle {
                        font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                        font_size: 50.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ScoreHudText,
            ));
        });
}

fn cleanup_score_hud(mut commands: Commands, text_query: Query<Entity, With<ScoreHudText>>) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_score_hud(mut text_query: Query<&mut Text, With<ScoreHudText>>, score: Res<Score>) {
    let score_value = (score.enemies_killed * 10) as i32
        - Instant::now()
            .duration_since(score.time_game_start)
            .as_secs() as i32;

    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!("Score: {}", score_value);
    }
}
