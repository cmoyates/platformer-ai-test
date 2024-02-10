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

use crate::{s_move_goal_point, GizmosVisible, Physics, GRAVITY_STRENGTH};

use super::{a_star::find_path, pathfinding::Pathfinding};

const WANDER_MAX_SPEED: f32 = 3.0;
// const PURSUE_MAX_SPEED: f32 = 5.0;
// const ATTACK_MAX_SPEED: f32 = 7.0;

// const STEERING_SCALE: f32 = 0.1;

pub const PLATFORMER_AI_JUMP_FORCE: f32 = 9.0;

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
        let (move_dir, jump_velocity) = get_move_inputs(
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

        // Jumping
        {
            // If the player is trying to jump
            if jump_velocity.length_squared() > 0.0 {
                // If on the ground
                if physics.grounded {
                    // Jump
                    physics.velocity = jump_velocity;
                    physics.grounded = false;
                    println!("Jump!!!");
                }
                // If on a wall
                else if physics.walled != 0 {
                    // Wall jump
                    physics.velocity = jump_velocity;
                    physics.walled = 0;
                    physics.has_wall_jumped = true;
                    println!("Wall Jump!!!");
                }
            }
        }

        update_physics_and_transform(&mut physics, &mut transform);
    }
}

fn get_move_inputs(
    pathfinding: &Pathfinding,
    current_position: Vec2,
    gizmos: &mut Gizmos,
    gizmos_visible: bool,
) -> (Vec2, Vec2) {
    let mut move_dir = Vec2::ZERO;
    let mut jump_velocity = Vec2::ZERO;

    let path = find_path(&pathfinding, current_position);

    if let Some(path) = path {
        if gizmos_visible {
            for i in 0..path.len() {
                gizmos.circle_2d(path[i].0, 5.0, Color::GREEN);
            }
        }

        if path.len() > 1 {
            let is_jumpable_connection = pathfinding.nodes[path[0].1]
                .jumpable_connections
                .contains(&path[1].1);

            if is_jumpable_connection {
                let delta_p = path[1].0 - path[0].0;
                let gravity_acceleration = Vec2::new(0.0, -0.5);
                let t_low_energy = 1.5
                    * (4.0 * delta_p.dot(delta_p) / gravity_acceleration.dot(gravity_acceleration))
                        .sqrt()
                        .sqrt();
                jump_velocity = delta_p / t_low_energy - gravity_acceleration * t_low_energy / 2.0;
            }

            move_dir = (path[1].0 - path[0].0).normalize();
        }
    }

    (move_dir, jump_velocity)
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
        physics.acceleration.y = -GRAVITY_STRENGTH;
    } else {
        let gravity_normal_dir = physics.normal * GRAVITY_STRENGTH;
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
