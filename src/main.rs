mod alien_ship;
mod alien_waves;
mod camera;
mod celestial_body;
mod collisions_handler;
mod death;
mod despawn_queue;
mod gravity;
mod healthpoints;
mod impulses_aggregator;
mod lasers;
mod menu;
mod player;
mod system_sets;
mod thruster;
mod ui;

use bevy::{log::LogPlugin, prelude::*};
use bevy_rapier2d::plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};
use bevy_vector_shapes::ShapePlugin;
use system_sets::AppStage;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Game,
    Menu,
    DeathScreen,
}

fn main() {
    let mut app = App::new();

    app.add_state::<AppState>();
    app.add_plugins((
        DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=error,wgpu_hal=error,space_chase=debug".into(),
            level: bevy::log::Level::DEBUG,
        }),
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0),
        ShapePlugin::default(),
    ));

    app.insert_resource(RapierConfiguration {
        gravity: Vec2::ZERO,
        timestep_mode: TimestepMode::Variable {
            max_dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 2,
        },
        ..default()
    });

    app.add_systems(
        OnEnter(AppState::Game),
        (
            ui::setup,
            player::setup,
            alien_waves::setup,
            celestial_body::setup,
        ),
    );
    app.add_systems(
        OnExit(AppState::Game),
        (
            player::cleanup,
            celestial_body::cleanup,
            alien_ship::cleanup,
        ),
    );

    app.add_systems(
        Update,
        (player::control, alien_waves::update, alien_ship::update)
            .in_set(AppStage::Control)
            .run_if(in_state(AppState::Game)),
    );
    app.add_systems(
        Update,
        (
            thruster::update,
            lasers::update,
            collisions_handler::update,
            gravity::update,
            celestial_body::update,
            death::update,
        )
            .in_set(AppStage::Simulation)
            .run_if(in_state(AppState::Game)),
    );
    app.add_systems(
        Update,
        (lasers::draw, ui::draw_healthbar, ui::draw_hud)
            .in_set(AppStage::Draw)
            .run_if(in_state(AppState::Game)),
    );
    impulses_aggregator::setup(&mut app);
    despawn_queue::setup(&mut app);
    system_sets::setup(&mut app);
    camera::setup(&mut app);
    menu::setup(&mut app);
    app.run();
}
