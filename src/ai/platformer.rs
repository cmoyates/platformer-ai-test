use bevy::{
    app::{App, Plugin, Update},
    ecs::{component::Component, schedule::IntoSystemConfigs, system::Query},
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};

use crate::{collisions::s_collision, Physics};

const WANDER_MAX_SPEED: f32 = 3.0;
const PURSUE_MAX_SPEED: f32 = 5.0;

const STEERING_SCALE: f32 = 0.1;

pub struct PlatformerAIPlugin;

impl Plugin for PlatformerAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_platformer_ai_movement.before(s_collision));
    }
}

#[derive(Component)]
pub struct PlatformerAI {}

pub fn s_platformer_ai_movement(
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PlatformerAI)>,
) {
    for (mut transform, mut physics, _) in platformer_ai_query.iter_mut() {
        physics.acceleration.y = -0.5;

        // Update velocity
        let new_velocity = physics.velocity + physics.acceleration;
        physics.velocity = new_velocity;

        // Update previous position
        physics.prev_position = transform.translation.xy();
        // Update position
        transform.translation.x += physics.velocity.x;
        transform.translation.y += physics.velocity.y;
    }
}
