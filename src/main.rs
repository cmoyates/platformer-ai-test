mod level;
mod utils;

use ::bevy::prelude::*;
use bevy::{
    app::AppExit,
    window::{PresentMode, PrimaryWindow},
};
use level::{generate_level_polygons, Polygon};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Platformer AI Test".to_string(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(Update, s_render)
        .run();
}

#[derive(Resource)]
pub struct Level {
    pub polygons: Vec<Polygon>,
    pub grid_size: f32,
    pub size: Vec2,
    pub half_size: Vec2,
}

pub fn s_init(mut commands: Commands) {
    let grid_size = 32.0;

    let (level_polygons, size, half_size) = generate_level_polygons(grid_size);

    commands.insert_resource(Level {
        polygons: level_polygons,
        grid_size,
        size,
        half_size,
    });

    commands.spawn(Camera2dBundle::default());
}

pub fn s_input(
    mouse_buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window>,
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        println!("Mouse button pressed");
        if let Some(position) = q_windows.single().cursor_position() {
            dbg!(position);
        }
    }

    // Escape to exit
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

pub fn s_render(mut gizmos: Gizmos, level: Res<Level>) {
    // Draw the level polygons
    for polygon in &level.polygons {
        gizmos.linestrip_2d(
            polygon.points.iter().cloned().collect::<Vec<Vec2>>(),
            polygon.color,
        );
    }
}
