use bevy::prelude::*;

use crate::impulses_aggregator::AddExternalImpulse;

#[derive(Component)]
pub struct Thruster {
    pub max_thrust: f32, // m/(kg*s^2)
    pub current_thrust: f32,
    pub rampup_rate: f32,
    pub shutoff_rate: f32,
    pub ignition_thrust: f32,
}

impl Thruster {
    pub fn throttle(&mut self, dt: f32) {
        self.current_thrust = if self.current_thrust < self.ignition_thrust {
            self.ignition_thrust
        } else {
            (self.current_thrust + self.rampup_rate * dt).min(self.max_thrust)
        }
    }

    pub fn release(&mut self, dt: f32) {
        self.current_thrust = (self.current_thrust - self.shutoff_rate * dt).max(0.0);
    }
}

pub fn update(
    mut impulses: EventWriter<AddExternalImpulse>,
    thrustables: Query<(Entity, &Transform, &Thruster)>,
) {
    for (entity, transform, thruster) in thrustables.iter() {
        let impulse = transform
            .up()
            .xy()
            .rotate(Vec2::new(thruster.current_thrust, 0.0));
        // debug!("Engine thrust: {}", thruster.current_thrust);
        impulses.send(AddExternalImpulse {
            entity,
            impulse,
            torque_impulse: 0.0,
        });
    }
}
