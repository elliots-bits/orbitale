use bevy::prelude::*;
use bevy_rapier2d::{dynamics::Velocity, geometry::ColliderMassProperties};

use crate::{celestial_body::CircularOrbitChain, impulses_aggregator::AddExternalImpulse};

const GRAVITATIONAL_CONSTANT: f32 = 32.0;

#[derive(Component)]
pub struct AttractingBody;

#[derive(Component, Default)]
pub struct AffectedByGravity {
    pub last_acceleration: Vec2,
}

#[inline(always)]
fn gravity_formula(d: f32, m: f32) -> f32 {
    // Real spacetime is very scary, empty, and difficult to navigate.
    // This one is a little bit more intuitive.
    GRAVITATIONAL_CONSTANT * m / d.max(1.0).powf(1.6)
}

pub fn update(
    mut _impulses: EventWriter<AddExternalImpulse>,
    time: Res<Time>,
    attracting_bodies: Query<(Entity, &ColliderMassProperties, &Transform), With<AttractingBody>>,
    mut affected_bodies: Query<(&mut Velocity, &Transform, &mut AffectedByGravity)>,
) {
    let mut attracting_pos_mass = Vec::<(Vec2, f32)>::new();
    for (entity, mass_props, transform) in attracting_bodies.iter() {
        if let &ColliderMassProperties::Mass(mass) = mass_props {
            attracting_pos_mass.push((transform.translation.xy(), mass));
        } else {
            error!("Attracting entity {:?} has a ColliderMassProperties that is not of the Mass variant. Can't compute gravity.", entity);
        }
    }
    for (
        mut velocity,
        Transform {
            translation: pos, ..
        },
        mut feedback,
    ) in affected_bodies.iter_mut()
    {
        let mut acceleration = Vec2::ZERO;
        for &(opos, omass) in attracting_pos_mass.iter() {
            let d = opos - pos.xy();
            acceleration += d.normalize() * gravity_formula(d.length(), omass);
        }
        feedback.last_acceleration = acceleration;
        velocity.linvel += acceleration * time.delta_seconds();
    }
}

pub struct CoursePlanning {
    pub path: Vec<(Vec2, f32)>, // (pos, distance to nearest object)
    pub closest_flyby: f32,
}

pub fn plan_course(
    max_dt: f32,
    step_dt: f32,
    mut pos: Vec2,
    mut velocity: Vec2,
    bodies: &Vec<(f32, f32, CircularOrbitChain)>, // (mass, radius, orbit)
) -> CoursePlanning {
    let mut t = 0.0;
    let mut path = vec![];
    let mut closest_flyby = f32::INFINITY;
    while t < max_dt {
        let mut acceleration = Vec2::ZERO;
        let mut closest_body_distance_at_step = f32::INFINITY;

        for (m, r, orbit) in bodies.iter() {
            let body_pos_at_time = orbit.pos(t);
            let d = body_pos_at_time - pos;
            if (d.length() - r) < closest_body_distance_at_step {
                closest_body_distance_at_step = d.length() - r;
            }
            acceleration += d.normalize() * gravity_formula(d.length(), *m);
        }

        velocity += acceleration * step_dt;
        pos += velocity * step_dt;
        path.push((pos, closest_body_distance_at_step));
        t += step_dt;

        if closest_body_distance_at_step < closest_flyby {
            closest_flyby = closest_body_distance_at_step;
        }
        if closest_flyby <= 0.0 {
            break;
        }
    }
    CoursePlanning {
        path,
        closest_flyby,
    }
}
