use bevy::prelude::*;
use bevy_rapier2d::geometry::ColliderMassProperties;

use crate::impulses_aggregator::AddExternalImpulse;

const GRAVITATIONAL_CONSTANT: f32 = 5.0e-2;

#[derive(Component)]
pub struct AttractingBody;

#[derive(Component)]
pub struct AffectedByGravity;

#[inline(always)]
fn gravity_formula(d: f32, m: f32) -> f32 {
    // Real spacetime is very scary, empty, and difficult to navigate.
    // This one is a little bit more intuitive.
    GRAVITATIONAL_CONSTANT * m / d.max(1.0).powf(1.5)
}

pub fn update(
    mut impulses: EventWriter<AddExternalImpulse>,
    attracting_bodies: Query<(Entity, &ColliderMassProperties, &Transform), With<AttractingBody>>,
    affected_bodies: Query<(Entity, &Transform, &ColliderMassProperties), With<AffectedByGravity>>,
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
        entity,
        Transform {
            translation: pos, ..
        },
        collider_mass_props,
    ) in affected_bodies.iter()
    {
        if let &ColliderMassProperties::Mass(mass) = collider_mass_props {
            let mut force = Vec2::ZERO;
            for &(opos, omass) in attracting_pos_mass.iter() {
                let d = opos - pos.xy();
                force += d.normalize() * gravity_formula(d.length(), omass);
            }
            impulses.send(AddExternalImpulse {
                entity,
                impulse: force * mass,
                torque_impulse: 0.0,
            });
        } else {
            error!("Attracted entity {:?} has a ColliderMassProperties that is not of the Mass variant. Can't compute gravity.", entity);
        }
    }
}

pub struct CoursePlanning {
    pub path: Vec<Vec2>,
    pub closest_flyby: f32,
}

pub fn plan_course(
    max_dt: f32,
    step_dt: f32,
    mut pos: Vec2,
    mut velocity: Vec2,
    bodies: Vec<(Vec2, f32, f32)>, // (position, mass)
) -> CoursePlanning {
    let mut t = 0.0;
    let mut path = vec![];
    let mut closest_flyby = f32::INFINITY;
    while t < max_dt {
        let mut acceleration = Vec2::ZERO;

        for (op, m, r) in bodies.iter() {
            let d = *op - pos;
            if (d.length() - r) < closest_flyby {
                closest_flyby = d.length() - r;
            }
            acceleration += d.normalize() * gravity_formula(d.length(), *m);
        }

        velocity += acceleration * step_dt;
        pos += velocity * step_dt;
        path.push(pos);
        t += step_dt;

        // if closest_flyby <= 0.0 {
        //     break;
        // }
    }
    CoursePlanning {
        path,
        closest_flyby,
    }
}
