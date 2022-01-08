use bevy::prelude::*;

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    App::new()
        .add_startup_system(setup_camera)
        .add_plugins(DefaultPlugins)
        .run();
}
