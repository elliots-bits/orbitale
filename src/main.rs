mod alien_ship;
mod alien_waves;
mod background;
mod camera;
mod collisions_handler;
mod despawn_queue;
mod impulses_aggregator;
mod lasers;
mod player;
mod system_sets;
mod thruster;

use bevy::{log::LogPlugin, prelude::*};
use bevy_rapier2d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use bevy_vector_shapes::ShapePlugin;
use system_sets::AppStage;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: "info,wgpu_core=error,wgpu_hal=error,space_chase=debug".into(),
        level: bevy::log::Level::DEBUG,
    }))
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
    .add_plugins(ShapePlugin::default())
    // .add_plugins(RapierDebugRenderPlugin::default())
    .add_systems(
        Startup,
        (
            camera::setup,
            player::setup,
            alien_waves::setup,
            background::setup,
        ),
    )
    .insert_resource(RapierConfiguration {
        gravity: Vec2::ZERO,
        timestep_mode: TimestepMode::Variable {
            max_dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 2,
        },
        ..default()
    });
    app.add_systems(
        Update,
        (player::control, alien_waves::update, alien_ship::update).in_set(AppStage::Control),
    );
    app.add_systems(
        Update,
        (thruster::update, lasers::update, collisions_handler::update).in_set(AppStage::Simulation),
    );
    app.add_systems(
        Update,
        (camera::update, lasers::draw, background::update).in_set(AppStage::Draw),
    );
    impulses_aggregator::setup(&mut app);
    despawn_queue::setup(&mut app);
    system_sets::setup(&mut app);
    app.run();
}
