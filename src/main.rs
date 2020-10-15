use bevy::prelude::*;

struct HeadMaterial(Handle<ColorMaterial>);
struct SnakeHead;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(HeadMaterial(
        materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
    ));
}

fn game_setup(mut commands: Commands, head_material: Res<HeadMaterial>) {
    commands
        .spawn(SpriteComponents {
            material: head_material.0,
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(SnakeHead);
}

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_default_plugins()
        .run();
}
