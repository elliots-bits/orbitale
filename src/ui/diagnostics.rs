use bevy::{diagnostic::DiagnosticsStore, prelude::*};

use crate::{alien_ship::AlienShipMarker, AppState};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

#[derive(Component)]
struct EntitiesCountText;

pub fn setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), setup_entities_count);
    app.add_systems(OnExit(AppState::Game), cleanup_score_hud);
    app.add_systems(
        Update,
        update_entities_count.run_if(in_state(AppState::Game)),
    );

    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
}

fn setup_entities_count(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Enemies: 0\n",
                TextStyle {
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ),
            TextSection::new(
                "Total entities: 0\n",
                TextStyle {
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ),
            TextSection::new(
                "FPS: N\\A",
                TextStyle {
                    font: asset_server.load("fusion-pixel-12px-proportional-latin.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        EntitiesCountText,
    ));
}

fn cleanup_score_hud(mut commands: Commands, text_query: Query<Entity, With<EntitiesCountText>>) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_entities_count(
    mut text_query: Query<&mut Text, With<EntitiesCountText>>,
    enemies_query: Query<Entity, With<AlienShipMarker>>,
    entities_query: Query<Entity>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!("Enemies: {}\n", enemies_query.iter().count());
        text.sections[1].value = format!("Total entities: {}\n", entities_query.iter().count());
        if let Some(fps) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[2].value = format!("FPS: {:.0}", fps);
        }
    }
}
