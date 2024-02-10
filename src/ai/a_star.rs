use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::math::Vec2;

use super::pathfinding::{Pathfinding, PathfindingGraphNode};

pub fn find_path(pathfinding: &Pathfinding, start_position: Vec2) -> Option<Vec<Vec2>> {
    if pathfinding.goal_graph_node.is_none() {
        return None;
    }

    let goal_node = pathfinding.goal_graph_node.as_ref().unwrap();

    let mut open_list: BinaryHeap<AStarNode> = BinaryHeap::new();
    let mut closed_list: Vec<AStarNode> = vec![];

    // Get the start node
    let start_node = get_start_node(pathfinding, start_position);

    // Add the start node to the open list
    open_list.push(start_node);

    loop {
        // If the open list is empty, there is no path
        if open_list.is_empty() {
            return None;
        }

        // Get the node with the lowest f-cost
        let current_node = open_list.pop().unwrap();

        // If the current node is the goal, reconstruct the path
        if current_node.is_goal {
            let mut path = vec![];

            let mut current_node = current_node;
            while let Some(parent_id) = current_node.parent {
                let parent_node = closed_list.iter().find(|n| n.id == parent_id).unwrap();
                path.push(parent_node.position);
                current_node = parent_node.clone();
            }

            path.reverse();

            return Some(path);
        }

        // If the node is in the closed list, skip it
        if closed_list.iter().any(|n| n.id == current_node.id) {
            continue;
        }

        // Add the current node to the closed list
        closed_list.push(current_node.clone());

        // For each connection of the current node
        for connection_id in current_node.connections.iter() {
            let connected_graph_node = &pathfinding.nodes[*connection_id];
            let mut new_node = AStarNode::new(connected_graph_node);

            // If the new node is the goal, set the is_goal flag
            if new_node.id == goal_node.id {
                new_node.is_goal = true;
            } else {
                // Set the g-cost to the distance to the start node
                new_node.g_cost =
                    (current_node.position - new_node.position).length() + current_node.g_cost;

                // Set the h-cost to the distance to the goal
                new_node.h_cost = (pathfinding.goal_position - new_node.position).length();
            }

            // Set the parent of the new node
            new_node.parent = Some(current_node.id);

            open_list.push(new_node);
        }
    }
}

fn get_start_node(pathfinding: &Pathfinding, start_position: Vec2) -> AStarNode {
    let mut start_graph_node: PathfindingGraphNode = PathfindingGraphNode {
        id: 0,
        position: Vec2::ZERO,
        polygon_index: 0,
        line_index: 0,
        connections: vec![],
    };
    let mut start_graph_node_index = f32::MAX;

    for node in pathfinding.nodes.iter() {
        let distance = (start_position - node.position).length_squared();

        if distance < start_graph_node_index {
            start_graph_node_index = distance;
            start_graph_node = node.clone();
        }
    }

    let mut start_a_star_node = AStarNode::new(&start_graph_node);

    // Set the h-cost to the distance to the goal
    start_a_star_node.h_cost = (pathfinding.goal_position - start_a_star_node.position).length();

    return start_a_star_node;
}

#[derive(Clone, Debug)]
pub struct AStarNode {
    pub position: Vec2,
    pub id: usize,
    pub connections: Vec<usize>,
    pub g_cost: f32,
    pub h_cost: f32,
    pub parent: Option<usize>,
    pub is_goal: bool,
}

impl AStarNode {
    pub fn new(graph_node: &PathfindingGraphNode) -> AStarNode {
        AStarNode {
            position: graph_node.position,
            id: graph_node.id,
            connections: graph_node.connections.clone(),
            g_cost: 0.0,
            h_cost: 0.0,
            parent: None,
            is_goal: false,
        }
    }

    pub fn get_f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_f_cost()
            .partial_cmp(&other.get_f_cost())
            .unwrap_or(Ordering::Equal)
            .reverse()
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
