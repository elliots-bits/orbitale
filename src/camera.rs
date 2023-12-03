use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use crate::player::PlayerMarker;

#[derive(Component)]
pub struct GameCameraMarker;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.1,
                    green: 0.1,
                    blue: 0.15,
                    alpha: 1.0,
                }),
            },
            ..default()
        },
        GameCameraMarker,
    ));
}

pub fn update(
    mut camera: Query<&mut Transform, (With<GameCameraMarker>, Without<PlayerMarker>)>,
    player: Query<&Transform, With<PlayerMarker>>,
) {
    if let Ok(mut cam_t) = camera.get_single_mut() {
        if let Ok(player_t) = player.get_single() {
            cam_t.translation = player_t.translation;
        }
    }
}
