mod level;
mod pathfinding;
mod utils;

use ::bevy::prelude::*;
use bevy::{app::AppExit, window::PresentMode};
use level::{generate_level_polygons, Polygon};
use pathfinding::{init_pathfinding, Pathfinding, PathfindingPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(InputDir { dir: Vec2::ZERO })
        .insert_resource(GizmosVisible { visible: false })
        .insert_resource(GoalPoint {
            position: Vec2::new(0.0, 0.0),
            enabled: false,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Platformer AI Test".to_string(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PathfindingPlugin)
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(Update, s_move_goal_point.after(s_input))
        .add_systems(Update, s_render.after(s_move_goal_point))
        .run();
}

#[derive(Resource)]
pub struct Level {
    pub polygons: Vec<Polygon>,
    pub grid_size: f32,
    pub size: Vec2,
    pub half_size: Vec2,
}

#[derive(Resource)]
pub struct InputDir {
    pub dir: Vec2,
}

#[derive(Resource)]
pub struct GizmosVisible {
    pub visible: bool,
}

#[derive(Resource)]
pub struct GoalPoint {
    pub position: Vec2,
    pub enabled: bool,
}

pub fn s_init(mut commands: Commands, pathfinding: ResMut<Pathfinding>) {
    let grid_size = 32.0;

    let (level_polygons, size, half_size) = generate_level_polygons(grid_size);

    let level = Level {
        polygons: level_polygons,
        grid_size,
        size,
        half_size,
    };

    init_pathfinding(&level, pathfinding);

    commands.insert_resource(level);

    commands.spawn(Camera2dBundle::default());
}

pub fn s_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut input_dir: ResMut<InputDir>,
    mut gizmos_visible: ResMut<GizmosVisible>,
    mut goal_point: ResMut<GoalPoint>,
) {
    // Escape to exit
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }

    // R to reset
    if keyboard_input.just_pressed(KeyCode::R) {
        println!("Reset");
    }

    // Arrow keys to move goal point
    {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::Up) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            direction.x += 1.0;
        }

        // Normalize direction
        direction = direction.normalize_or_zero();

        // Set direction resource
        input_dir.dir = direction;
    }

    // G to toggle gizmos
    if keyboard_input.just_pressed(KeyCode::G) {
        gizmos_visible.visible = !gizmos_visible.visible;
    }

    // Space to toggle goal point
    if keyboard_input.just_pressed(KeyCode::Space) {
        goal_point.enabled = !goal_point.enabled;
    }
}

pub fn s_move_goal_point(input_dir: Res<InputDir>, mut goal_point: ResMut<GoalPoint>) {
    goal_point.position += input_dir.dir * 4.0;
}

pub fn s_render(
    mut gizmos: Gizmos,
    level: Res<Level>,
    goal_point: Res<GoalPoint>,
    pathfinding: Res<Pathfinding>,
    gizmos_visible: Res<GizmosVisible>,
) {
    // Draw the level polygons
    for polygon in &level.polygons {
        gizmos.linestrip_2d(
            polygon.points.iter().cloned().collect::<Vec<Vec2>>(),
            polygon.color,
        );
    }

    if gizmos_visible.visible {
        // Draw the pathfinding nodes
        for node in &pathfinding.nodes {
            gizmos.circle_2d(node.position, 2.5, Color::WHITE);
        }
    }

    // Draw the goal point
    gizmos.circle_2d(
        goal_point.position,
        7.5,
        Color::GREEN.with_a(if goal_point.enabled { 1.0 } else { 0.1 }),
    );
}
