use bevy::prelude::*;
use bevy_rapier2d::{
    dynamics::Velocity,
    geometry::{Collider, ColliderMassProperties},
};

use crate::{
    alien_ship::AlienShipMarker,
    celestial_body::{CelestialBodyMarker, CircularOrbitChain},
    gravity::plan_course,
    player::PlayerMarker,
};

pub const PLAYER_PLAN_DURATION: f32 = 30.0;
pub const PLAYER_PLAN_STEP_DT: f32 = 0.05;

const MAX_ENEMY_TRAJECTORIES_COMPUTED_PER_FRAME: u32 = 60;
const STALE_TRAJECTORY_AGE: f32 = 5.0;
const ENEMY_PLAN_DURATION: f32 = 10.0;
const ENEMY_PLAN_STEP_DT: f32 = 0.5;

#[derive(Component)]
pub struct ComputedTrajectory {
    pub computation_requested: bool,
    pub computed_at: f32,
    pub step_dt: f32,
    pub path: Vec<(Vec2, f32)>, // (path, distance to nearest object)
    pub closest_flyby: f32,     // will be 0.0 if the trajectory is a collision course
}

impl Default for ComputedTrajectory {
    fn default() -> Self {
        ComputedTrajectory {
            computation_requested: true,
            computed_at: 0.0,
            step_dt: 0.0,
            path: vec![],
            closest_flyby: f32::INFINITY,
        }
    }
}

pub fn compute_player_trajectory(
    time: Res<Time>,
    mut player: Query<(&Transform, &Velocity, &mut ComputedTrajectory), With<PlayerMarker>>,
    bodies: Query<
        (
            &Transform,
            &ColliderMassProperties,
            &Collider,
            &CircularOrbitChain,
        ),
        With<CelestialBodyMarker>,
    >,
) {
    if let Ok((t, v, mut traj)) = player.get_single_mut() {
        if traj.computation_requested {
            let planned_course = plan_course(
                PLAYER_PLAN_DURATION,
                PLAYER_PLAN_STEP_DT,
                t.translation.xy(),
                v.linvel,
                &collect_celestial_bodies(bodies),
            );
            traj.computed_at = time.elapsed_seconds();
            traj.step_dt = PLAYER_PLAN_STEP_DT;
            traj.path = planned_course.path;
            traj.closest_flyby = planned_course.closest_flyby;
        }
    }
}

pub fn compute_enemies_trajectories(
    time: Res<Time>,
    mut ships: Query<
        (Entity, &Transform, &Velocity, &mut ComputedTrajectory),
        With<AlienShipMarker>,
    >,
    bodies: Query<
        (
            &Transform,
            &ColliderMassProperties,
            &Collider,
            &CircularOrbitChain,
        ),
        With<CelestialBodyMarker>,
    >,
) {
    let bodies = collect_celestial_bodies(bodies);

    let mut total_computed = 0u32;
    for (_, t, v, mut traj) in ships.iter_mut() {
        if total_computed >= MAX_ENEMY_TRAJECTORIES_COMPUTED_PER_FRAME {
            break;
        }
        if traj.computation_requested
            && time.elapsed_seconds() - traj.computed_at >= STALE_TRAJECTORY_AGE
        {
            let planned_course = plan_course(
                ENEMY_PLAN_DURATION,
                ENEMY_PLAN_STEP_DT,
                t.translation.xy(),
                v.linvel,
                &bodies,
            );
            traj.computed_at = time.elapsed_seconds();
            traj.step_dt = ENEMY_PLAN_STEP_DT;
            traj.path = planned_course.path;
            traj.closest_flyby = planned_course.closest_flyby;
            total_computed += 1;
        }
    }
}

fn collect_celestial_bodies(
    q: Query<
        (
            &Transform,
            &ColliderMassProperties,
            &Collider,
            &CircularOrbitChain,
        ),
        With<CelestialBodyMarker>,
    >,
) -> Vec<(f32, f32, CircularOrbitChain)> {
    q.iter()
        .map(|(_, massprops, collider, circular_orbit)| {
            if let ColliderMassProperties::Mass(mass) = massprops {
                (
                    *mass,
                    collider.as_ball().unwrap().radius(),
                    circular_orbit.clone(),
                )
            } else {
                panic!("Use ColliderMassProperties::Mass(x) for celestial bodies")
            }
        })
        .collect::<Vec<(f32, f32, CircularOrbitChain)>>()
}
