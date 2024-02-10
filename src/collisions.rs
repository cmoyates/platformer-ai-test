use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        schedule::IntoSystemConfigs,
        system::{Query, Res},
    },
    gizmos::gizmos::Gizmos,
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};

use crate::{
    ai::platformer_ai::s_platformer_ai_movement,
    utils::{line_intersect, side_of_line_detection},
    Level, Physics,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_collision.after(s_platformer_ai_movement));
    }
}

pub fn s_collision(
    mut entity_query: Query<(&mut Transform, &mut Physics)>,
    level: Res<Level>,
    mut gizmos: Gizmos,
) {
    if let Ok((mut transform, mut physics)) = entity_query.get_single_mut() {
        let mut adjustment = Vec2::ZERO;
        let mut new_normal = Vec2::ZERO;

        for polygon_index in 0..level.polygons.len() {
            let polygon = level.polygons.get(polygon_index).unwrap();

            let mut intersect_counter = 0;
            let mut colliding_with_polygon = false;

            for line_index in 1..polygon.points.len() {
                let start = polygon.points[line_index - 1];
                let end = polygon.points[line_index];

                // Intersection detection
                {
                    let intersection = line_intersect(
                        start,
                        end,
                        transform.translation.xy(),
                        transform.translation.xy() + Vec2::new(2.0, 1.0) * 10000.0,
                    );

                    if intersection.is_some() {
                        intersect_counter += 1;
                    }
                }

                let previous_side_of_line =
                    side_of_line_detection(start, end, physics.prev_position);

                if previous_side_of_line != 1.0 {
                    continue;
                }

                let (distance_sq, projection) =
                    find_projection(start, end, transform.translation.xy(), physics.radius);

                let colliding_with_line = distance_sq <= physics.radius.powi(2);
                colliding_with_polygon = colliding_with_polygon || colliding_with_line;

                let touch_radius = physics.radius + 0.5;

                let touching_line = distance_sq <= touch_radius.powi(2);

                if touching_line {
                    let normal_dir = (transform.translation.xy() - projection).normalize_or_zero();

                    // If the line is not above the player
                    if normal_dir.y >= -0.01 {
                        // Add the normal dir to the players new normal
                        new_normal -= normal_dir;

                        // If the player is on a wall
                        if normal_dir.x.abs() >= 0.8 {
                            physics.walled = normal_dir.x.signum() as i8;
                            physics.has_wall_jumped = false;
                        }

                        // If the player is on the ground
                        if normal_dir.y > 0.01 {
                            physics.grounded = true;
                            physics.walled = 0;
                            physics.has_wall_jumped = false;
                        }
                    }
                }

                if colliding_with_line {
                    let mut delta = (transform.translation.xy() - projection).normalize_or_zero();

                    if delta.y < -0.01 {
                        // println!("Hit ceiling");
                        physics.velocity.y = 0.0;
                    }

                    delta *= physics.radius - distance_sq.sqrt();

                    if delta.x.abs() > adjustment.x.abs() {
                        adjustment.x = delta.x;
                    }
                    if delta.y.abs() > adjustment.y.abs() {
                        adjustment.y = delta.y;
                    }
                }
            }

            let inside_polygon = intersect_counter % 2 == 1;

            if colliding_with_polygon && inside_polygon && !polygon.is_container {
                transform.translation = physics.prev_position.extend(0.0);
            }
        }

        // Update the players normal
        new_normal = new_normal.normalize_or_zero();
        physics.normal = new_normal;

        // Remove the players velocity in the direction of the normal
        let velocity_adjustment = physics.velocity.dot(new_normal) * new_normal;

        physics.velocity -= velocity_adjustment;

        // Update the players position
        transform.translation += adjustment.extend(0.0);
    }
}

pub fn find_projection(start: Vec2, end: Vec2, point: Vec2, radius: f32) -> (f32, Vec2) {
    let point_vec = point - start;
    let line_vec = end - start;

    let line_vec_normalized = line_vec.normalize();

    let dot = point_vec.dot(line_vec_normalized);

    let projection_point = line_vec_normalized * dot + start;

    // If the projection point is outside the line past start
    if (end - projection_point).length_squared() > (end - start).length_squared() {
        return (point_vec.length_squared() + radius * 2.0, start);
    }
    // If the projection point is outside the line past end
    if (projection_point - start).length_squared() > (end - start).length_squared() {
        return ((point - end).length_squared() + radius * 2.0, end);
    }

    let dist = (point - projection_point).length_squared();

    return (dist, projection_point);
}
