use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    gizmos::gizmos::Gizmos,
    math::{Vec2, Vec3Swizzles},
    render::color::Color,
    transform::components::Transform,
};

use crate::{s_move_goal_point, GizmosVisible, Physics};

use super::{a_star::find_path, pathfinding::Pathfinding};

const WANDER_MAX_SPEED: f32 = 3.0;
// const PURSUE_MAX_SPEED: f32 = 5.0;
// const ATTACK_MAX_SPEED: f32 = 7.0;

// const STEERING_SCALE: f32 = 0.1;

pub const ACCELERATION_SCALERS: (f32, f32) = (0.2, 0.4);

pub struct PlatformerAIPlugin;

impl Plugin for PlatformerAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_platformer_ai_movement.after(s_move_goal_point));
    }
}

#[derive(Component)]
pub struct PlatformerAI {}

pub fn s_platformer_ai_movement(
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PlatformerAI)>,
    pathfinding: Res<Pathfinding>,
    gismo_visible: Res<GizmosVisible>,
    mut gizmos: Gizmos,
) {
    for (mut transform, mut physics, mut platformer_ai) in platformer_ai_query.iter_mut() {
        let move_dir = get_move_dir(
            pathfinding.as_ref(),
            transform.translation.xy(),
            &mut gizmos,
            gismo_visible.visible,
        );

        if gismo_visible.visible {
            gizmos.line_2d(
                transform.translation.xy(),
                transform.translation.xy() + move_dir * 15.0,
                Color::RED,
            );
        }

        let falling = physics.normal.length_squared() == 0.0;
        let no_move_dir = move_dir.length_squared() == 0.0;

        apply_movement_acceleration(&mut physics, &move_dir, falling, no_move_dir);

        apply_gravity_toward_normal(&mut physics, falling /*, player_move_off_wall*/);

        update_physics_and_transform(&mut physics, &mut transform);
    }
}

fn get_move_dir(
    pathfinding: &Pathfinding,
    current_position: Vec2,
    gizmos: &mut Gizmos,
    gizmos_visible: bool,
) -> Vec2 {
    let mut move_dir = Vec2::ZERO;

    let path = find_path(&pathfinding, current_position);

    if let Some(path) = path {
        if gizmos_visible {
            for i in 0..path.len() {
                gizmos.circle_2d(path[i], 5.0, Color::GREEN);
            }
        }

        if path.len() > 1 {
            move_dir = (path[1] - path[0]).normalize();
        }
    }

    move_dir
}

fn apply_movement_acceleration(
    physics: &mut Physics,
    move_dir: &Vec2,
    falling: bool,
    no_move_dir: bool,
) {
    // Apply acceleration
    physics.acceleration = (*move_dir * WANDER_MAX_SPEED - physics.velocity)
        * if no_move_dir {
            // Deacceleration
            ACCELERATION_SCALERS.1
        } else {
            // Acceleration
            ACCELERATION_SCALERS.0
        };

    // If the player is falling
    if falling {
        // Ignore any other acceleration in the y direction
        physics.acceleration.y = 0.0;
    }
    // // Unless the player is on a wall and is trying to move away from it
    // if !player_move_off_wall {
    //     // Remove the acceleration in the direction of the normal
    //     let acceleration_adjustment =
    //         player_physics.normal * player_physics.acceleration.dot(player_physics.normal);
    //     player_physics.acceleration -= acceleration_adjustment;
    // }
}

fn apply_gravity_toward_normal(
    physics: &mut Physics,
    falling: bool,
    // player_move_off_wall: bool,
) {
    if
    /*player_move_off_wall || */
    falling {
        physics.acceleration.y = -0.5;
    } else {
        let gravity_normal_dir = physics.normal * 0.5;
        physics.acceleration += gravity_normal_dir;
    }
}

fn update_physics_and_transform(physics: &mut Physics, transform: &mut Transform) {
    // Update velocity
    let new_velocity = physics.velocity + physics.acceleration;
    physics.velocity = new_velocity;

    // Update previous position
    physics.prev_position = transform.translation.xy();
    // Update position
    transform.translation.x += physics.velocity.x;
    transform.translation.y += physics.velocity.y;
}
