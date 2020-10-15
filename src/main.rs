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

fn snake_movement(mut head_positions: Query<(&SnakeHead, &mut Transform)>) {
    for (_head, mut transform) in &mut head_positions.iter() {
        *transform.translation_mut().y_mut() += 10.;
    }
}

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(snake_movement.system())
        .add_default_plugins()
        .run();
}
