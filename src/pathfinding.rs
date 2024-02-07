use bevy::{
    app::{App, Plugin},
    ecs::system::{ResMut, Resource},
    math::Vec2,
};

use crate::Level;

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Pathfinding { nodes: Vec::new() });
    }
}

pub fn init_pathfinding(level: &Level, mut pathfinding: ResMut<Pathfinding>) {
    let mut outer_container_seen = false;

    for polygon_index in 0..level.polygons.len() {
        let polygon = &level.polygons[polygon_index];
        if polygon.is_container {
            outer_container_seen = !outer_container_seen;
        }

        if outer_container_seen && polygon.is_container {
            continue;
        }

        for line_index in 1..polygon.points.len() {
            let start = polygon.points[line_index - 1];
            let end = polygon.points[line_index];

            let mut start_to_end = end - start;

            let length = start_to_end.length();

            let nodes_on_line_count = (length.abs() / 20.0).ceil() as f32;
            let dist_between_nodes_on_line = length / nodes_on_line_count;

            start_to_end = start_to_end.normalize();

            if start_to_end.dot(Vec2::X) > -0.1 {
                for j in 0..(nodes_on_line_count as i32) {
                    let node_pos = start + start_to_end * (j as f32 * dist_between_nodes_on_line);
                    pathfinding.nodes.push(PathfindingNode {
                        position: node_pos,
                        polygon_index: polygon_index,
                        line_index: line_index - 1,
                    });
                }
                pathfinding.nodes.push(PathfindingNode {
                    position: end,
                    polygon_index: polygon_index,
                    line_index: line_index - 1,
                });
            }
        }
    }
}

pub struct PathfindingNode {
    pub position: Vec2,
    pub polygon_index: usize,
    pub line_index: usize,
}

#[derive(Resource)]
pub struct Pathfinding {
    pub nodes: Vec<PathfindingNode>,
}
