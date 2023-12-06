use bevy::prelude::*;
use bevy_rapier2d::geometry::ColliderMassProperties;

use crate::impulses_aggregator::AddExternalImpulse;

const GRAVITATIONAL_CONSTANT: f32 = 1.0;

#[derive(Component)]
pub struct AttractingBody;

#[derive(Component)]
pub struct AffectedByGravity;

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
                force += d.normalize() * GRAVITATIONAL_CONSTANT * omass / d.length().powf(2.0);
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
