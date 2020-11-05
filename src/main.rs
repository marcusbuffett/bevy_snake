use bevy::prelude::*;

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_plugins(DefaultPlugins)
        .run();
}
