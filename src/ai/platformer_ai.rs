use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_3d::graph::node,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathFollowingStrategy {
    CurrentNodeToNextNode,
    CurrentNodeOffsetToNextNodeOffset,
    AgentToCurrentNode,
    AgentToCurrentNodeOffset,
    AgentToNextNode,
    AgentToNextNodeOffset,
    AgentToGoal,
    None,
}

pub const PLATFORMER_AI_AGENT_RADIUS: f32 = 8.0;

const WANDER_MAX_SPEED: f32 = 3.0;
// const PURSUE_MAX_SPEED: f32 = 5.0;
// const ATTACK_MAX_SPEED: f32 = 7.0;

// const STEERING_SCALE: f32 = 0.1;

pub const PLATFORMER_AI_JUMP_FORCE: f32 = 8.0;

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
        let (move_dir, jump_velocity, jump_from_node, jump_to_node) = get_move_inputs(
            pathfinding.as_ref(),
            transform.translation.xy(),
            &physics,
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

                    println!("Jump!!!");
                    platformer_ai.jump_from_pos = jump_from_node;
                    platformer_ai.jump_to_pos = jump_to_node;
                    println!("Initial Jump Velocity: {}", jump_velocity.length());
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
                    println!("Wall Jump!!!");
                    dbg!(move_dir);
                    dbg!(transform.translation.xy());
                    platformer_ai.jump_from_pos = jump_from_node;
                    platformer_ai.jump_to_pos = jump_to_node;
                    println!("Initial Jump Velocity: {}", jump_velocity.length());
                }
            }
        }

        update_physics_and_transform(&mut physics, &mut transform);

        // dbg!(physics.velocity);
    }
}

fn get_move_inputs(
    pathfinding: &Pathfinding,
    agent_position: Vec2,
    agent_physics: &Physics,
    gizmos: &mut Gizmos,
    gizmos_visible: bool,
) -> (Vec2, Vec2, Option<Vec2>, Option<Vec2>) {
    let mut move_dir = Vec2::ZERO;
    let mut jump_velocity = Vec2::ZERO;
    let mut jump_from_node = None;
    let mut jump_to_node = None;

    let path = find_path(&pathfinding, agent_position);

    if let Some(path) = path {
        if gizmos_visible {
            let mut prev_pos = agent_position;
            for i in 0..path.len() {
                gizmos.circle_2d(path[i].position, 5.0, Color::GREEN);
                gizmos.line_2d(prev_pos, path[i].position, Color::GREEN);

                prev_pos = path[i].position;
            }

            gizmos.line_2d(prev_pos, pathfinding.goal_position, Color::GREEN);
        }

        if path.len() > 1 {
            let offset_current_node =
                path[0].position + pathfinding.nodes[path[0].id].normal * agent_physics.radius;
            let offset_next_node: Vec2 =
                path[1].position + pathfinding.nodes[path[1].id].normal * agent_physics.radius;

            let agent_on_wall = agent_physics.normal.y > -0.01;

            let corner_is_external = pathfinding.nodes[path[0].id].is_external_corner;

            let current_node_is_corner = corner_is_external.is_some();

            let is_jumpable_connection = pathfinding.nodes[path[0].id]
                .jumpable_connections
                .iter()
                .any(|jumpable_connection| jumpable_connection.node_id == path[1].id);

            let falling = agent_physics.normal.length_squared() <= 0.0;

            let path_following_strategy: PathFollowingStrategy;

            // Agent not falling
            if !falling {
                // Agent jumping
                if is_jumpable_connection {
                    let agent_on_other_side_next_frame = agent_on_other_side_next_frame(
                        agent_position,
                        agent_physics.velocity,
                        path[0].position,
                        agent_on_wall,
                    );

                    let agent_not_moving = agent_physics.velocity.length_squared() < 0.1;

                    path_following_strategy = if agent_on_other_side_next_frame || agent_not_moving
                    {
                        println!("Test 1");
                        PathFollowingStrategy::AgentToNextNodeOffset
                    } else {
                        println!("Test 2");
                        PathFollowingStrategy::AgentToCurrentNodeOffset
                    };
                } else {
                    // Non-jumping corner
                    if current_node_is_corner {
                        println!("Test 3");
                        path_following_strategy = PathFollowingStrategy::AgentToNextNode;
                    }
                    // Non-jumping flat surface
                    else {
                        let current_pos_to_next_offset = offset_next_node - agent_position;
                        let current_offset_to_next_offset = offset_next_node - offset_current_node;

                        if current_pos_to_next_offset.length_squared()
                            <= current_offset_to_next_offset.length_squared()
                        {
                            println!("Test 4");
                            path_following_strategy = PathFollowingStrategy::AgentToNextNodeOffset;
                        } else {
                            println!("Test 5");
                            path_following_strategy =
                                PathFollowingStrategy::AgentToCurrentNodeOffset;
                        }
                    }
                }
            }
            // Agent falling
            else {
                println!("Test 6");
                path_following_strategy = PathFollowingStrategy::AgentToNextNodeOffset;
            }

            move_dir = match path_following_strategy {
                PathFollowingStrategy::CurrentNodeToNextNode => path[1].position - path[0].position,
                PathFollowingStrategy::CurrentNodeOffsetToNextNodeOffset => {
                    offset_next_node - offset_current_node
                }
                PathFollowingStrategy::AgentToCurrentNode => path[0].position - agent_position,
                PathFollowingStrategy::AgentToCurrentNodeOffset => {
                    offset_current_node - agent_position
                }
                PathFollowingStrategy::AgentToNextNode => path[1].position - agent_position,
                PathFollowingStrategy::AgentToNextNodeOffset => offset_next_node - agent_position,
                PathFollowingStrategy::AgentToGoal => pathfinding.goal_position - agent_position,
                PathFollowingStrategy::None => Vec2::ZERO,
                _ => Vec2::ZERO,
            }
            .normalize_or_zero();

            // Jumping
            if path_following_strategy == PathFollowingStrategy::AgentToNextNodeOffset
                || path_following_strategy == PathFollowingStrategy::AgentToNextNode
            {
                if is_jumpable_connection {
                    let node_position_delta = path[1].position - path[0].position;
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

pub fn agent_on_other_side_next_frame(
    agent_position: Vec2,
    agent_velocity: Vec2,
    node_position: Vec2,
    vertical: bool,
) -> bool {
    let dimension_index = if vertical { 1 } else { 0 };

    let agent_position_next_frame = agent_position + agent_velocity;

    let agent_side_of_corner_current =
        (agent_position[dimension_index] - node_position[dimension_index]).signum();

    let agent_side_of_corner_next_frame =
        (agent_position_next_frame[dimension_index] - node_position[dimension_index]).signum();

    agent_side_of_corner_current != agent_side_of_corner_next_frame
}
