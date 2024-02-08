use std::path;

use bevy::{
    app::{App, Plugin},
    ecs::system::{ResMut, Resource},
    math::Vec2,
};

use crate::{level::PolygonLine, Level};

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Pathfinding { nodes: Vec::new() });
    }
}

pub fn init_pathfinding(level: &Level, mut pathfinding: ResMut<Pathfinding>) {
    let mut outer_container_seen = false;

    // Place nodes
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
                let polygon_line = PolygonLine {
                    polygon_index: polygon_index,
                    line_index: line_index - 1,
                };

                for j in 0..(nodes_on_line_count as i32) {
                    let node_pos = start + start_to_end * (j as f32 * dist_between_nodes_on_line);

                    let mut new_node = PathfindingNode {
                        id: pathfinding.nodes.len(),
                        position: node_pos,
                        polygon_index: polygon_index,
                        line_index: line_index - 1,
                        connections: Vec::new(),
                        on_lines: vec![polygon_line.clone()],
                    };

                    if j > 0 {
                        new_node.connections.push(pathfinding.nodes.len() - 1);
                    }

                    pathfinding.nodes.push(new_node);
                }
                let new_node = PathfindingNode {
                    id: pathfinding.nodes.len(),
                    position: end,
                    polygon_index: polygon_index,
                    line_index: line_index - 1,
                    connections: vec![pathfinding.nodes.len() - 1],
                    on_lines: vec![polygon_line],
                };

                pathfinding.nodes.push(new_node);
            }
        }
    }

    // Make all connections 2-way
    for node_index in 0..pathfinding.nodes.len() {
        // Make a clone of the current node to appease the borrow checker
        let node = pathfinding.nodes[node_index].clone();

        // For each connection of the current node
        for other_node_index in node.connections.iter() {
            // Add the current node to the connections of the other node
            pathfinding.nodes[*other_node_index]
                .connections
                .push(node_index);
        }
    }

    // Remove duplicate nodes
    {
        let mut i = 0;
        while i < pathfinding.nodes.len() {
            let mut j = i + 1;
            while j < pathfinding.nodes.len() {
                if (pathfinding.nodes[i].position - pathfinding.nodes[j].position).length_squared()
                    < 1.0
                {
                    // Append the lines to the first node
                    let mut j_on_lines = pathfinding.nodes[j].on_lines.clone();
                    pathfinding.nodes[i].on_lines.append(&mut j_on_lines);

                    // Append the connections to the first node
                    let mut j_connections = pathfinding.nodes[j].connections.clone();
                    pathfinding.nodes[i].connections.append(&mut j_connections);

                    // Record the id of the nodes
                    let first_node_id = pathfinding.nodes[i].id;
                    let second_node_id = pathfinding.nodes[j].id;

                    // Remove the second node
                    pathfinding.nodes.remove(j);

                    // Update the connections of the nodes that were connected to the second node
                    for node in &mut pathfinding.nodes {
                        for connection in &mut node.connections {
                            if *connection == second_node_id {
                                *connection = first_node_id;
                            }
                        }
                    }
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    // Update the ids and connections to reflect the indices of the nodes
    {
        let pathfinding_nodes_copy = pathfinding.nodes.clone();

        for node_index in 0..pathfinding.nodes.len() {
            pathfinding.nodes[node_index].id = node_index;

            for connection_index in 0..pathfinding.nodes[node_index].connections.len() {
                let connected_node = pathfinding_nodes_copy
                    .iter()
                    .find(|n| n.id == pathfinding.nodes[node_index].connections[connection_index])
                    .unwrap();

                let connected_node_index = pathfinding_nodes_copy
                    .iter()
                    .position(|n| n.id == connected_node.id)
                    .unwrap();

                pathfinding.nodes[node_index].connections[connection_index] = connected_node_index;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathfindingNode {
    pub id: usize,
    pub position: Vec2,
    pub polygon_index: usize,
    pub line_index: usize,
    pub connections: Vec<usize>,
    pub on_lines: Vec<PolygonLine>,
}

#[derive(Resource)]
pub struct Pathfinding {
    pub nodes: Vec<PathfindingNode>,
}
