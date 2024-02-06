use ::bevy::prelude::*;
use bevy::window::PresentMode;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flee AI Test".to_string(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .run();
}

pub fn s_init(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
