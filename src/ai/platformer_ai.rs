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
pub struct PlatformerAI {
    pub current_target_node: Option<usize>,
    pub jump_from_pos: Option<Vec2>,
    pub jump_to_pos: Option<Vec2>,
}

pub fn s_platformer_ai_movement(
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PlatformerAI)>,
    pathfinding: Res<Pathfinding>,
    gismo_visible: Res<GizmosVisible>,
    mut gizmos: Gizmos,
) {
    for (mut transform, mut physics, mut platformer_ai) in platformer_ai_query.iter_mut() {
        let (move_dir, jump_velocity, jump_from_node, jump_to_node) =
            if physics.rememebered_move_dir.is_some() {
                (
                    physics.rememebered_move_dir.unwrap(),
                    Vec2::new(0.0, PLATFORMER_AI_JUMP_FORCE),
                    None,
                    None,
                )
            } else {
                get_move_inputs(
                    pathfinding.as_ref(),
                    transform.translation.xy(),
                    &mut gizmos,
                    gismo_visible.visible,
                    physics.radius,
                )
            };

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
            if jump_velocity.length_squared() > 0.0 && !falling {
                // If on the ground
                if physics.grounded {
                    // Jump
                    physics.velocity = jump_velocity;
                    physics.acceleration.x = 0.0;
                    physics.acceleration.y = -GRAVITY_STRENGTH;
                    physics.grounded = false;
                    physics.has_wall_jumped = false;
                    physics.walled = 0;

                    // physics.rememebered_move_dir = Some(if move_dir.dot(Vec2::X).signum() == 1.0 {
                    //     Vec2::X
                    // } else {
                    //     -Vec2::X
                    // });
                    println!("Jump!!!");
                }
                // If on a wall
                else if physics.walled != 0 {
                    // Wall jump
                    physics.velocity = jump_velocity;
                    physics.acceleration.x = 0.0;
                    physics.acceleration.y = -GRAVITY_STRENGTH;
                    physics.walled = 0;
                    physics.grounded = false;
                    physics.has_wall_jumped = true;
                    physics.rememebered_move_dir = Some(if move_dir.dot(Vec2::X).signum() == 1.0 {
                        Vec2::X
                    } else {
                        -Vec2::X
                    });
                    println!("Wall Jump!!!");
                    dbg!(move_dir);
                    dbg!(transform.translation.xy());
                    platformer_ai.jump_from_pos = jump_from_node;
                    platformer_ai.jump_to_pos = jump_to_node;
                }
            }
        }

        update_physics_and_transform(&mut physics, &mut transform);

        // dbg!(physics.velocity);
    }
}

fn get_move_inputs(
    pathfinding: &Pathfinding,
    current_position: Vec2,
    gizmos: &mut Gizmos,
    gizmos_visible: bool,
    radius: f32,
) -> (Vec2, Vec2, Option<Vec2>, Option<Vec2>) {
    let mut move_dir = Vec2::ZERO;
    let mut jump_velocity = Vec2::ZERO;
    let mut jump_from_node = None;
    let mut jump_to_node = None;

    let path = find_path(&pathfinding, current_position);

    if let Some(path) = path {
        if gizmos_visible {
            let mut prev_pos = current_position;
            for i in 0..path.len() {
                gizmos.circle_2d(path[i].0, 5.0, Color::GREEN);
                gizmos.line_2d(prev_pos, path[i].0, Color::GREEN);

                prev_pos = path[i].0;
            }
            gizmos.line_2d(prev_pos, pathfinding.goal_position, Color::GREEN);
        }

        if path.len() > 1 {
            let offset_current_node = path[0].0 + pathfinding.nodes[path[0].1].normal * radius;
            let offset_next_node = path[1].0 + pathfinding.nodes[path[1].1].normal * radius;

            let current_to_next_node = offset_next_node - current_position;
            let current_node_to_next_node = offset_next_node - offset_current_node;

            let targeting_next_node =
                current_to_next_node.length_squared() <= current_node_to_next_node.length_squared();

            let target_pos = if targeting_next_node {
                offset_next_node
            } else {
                offset_current_node
            };

            if gizmos_visible {
                gizmos.circle_2d(target_pos, 5.0, Color::PURPLE);
            }

            move_dir = (target_pos - current_position).normalize_or_zero();

            if targeting_next_node {
                let is_jumpable_connection = pathfinding.nodes[path[0].1]
                    .jumpable_connections
                    .contains(&path[1].1);

                if is_jumpable_connection {
                    let node_position_delta = path[1].0 - path[0].0;
                    let gravity_acceleration = Vec2::new(0.0, -GRAVITY_STRENGTH);
                    let jump_time = 1.0
                        * (4.0 * node_position_delta.dot(node_position_delta)
                            / gravity_acceleration.dot(gravity_acceleration))
                        .sqrt()
                        .sqrt();
                    jump_velocity =
                        node_position_delta / jump_time - gravity_acceleration * jump_time / 2.0;

                    jump_from_node = Some(offset_current_node);
                    jump_to_node = Some(offset_next_node);
                }
            }
        }
    }

    (move_dir, jump_velocity, jump_from_node, jump_to_node)
}

fn apply_movement_acceleration(
    physics: &mut Physics,
    move_dir: &Vec2,
    falling: bool,
    no_move_dir: bool,
) {
    // If the player is falling
    if falling {
        physics.acceleration = Vec2::ZERO;
        return;
    }

    // Apply acceleration
    physics.acceleration = (*move_dir * WANDER_MAX_SPEED - physics.velocity)
        * if no_move_dir {
            // Deacceleration
            ACCELERATION_SCALERS.1
        } else {
            // Acceleration
            ACCELERATION_SCALERS.0
        };

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
