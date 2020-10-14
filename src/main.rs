use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::collections::hash_set::HashSet;
use std::time::Duration;

pub struct HelloPlugin;

const WIDTH: u32 = 40;
const HEIGHT: u32 = 40;
const SNAKE_SPEED: f64 = 1. / 15.;
const SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const SEGMENT_SIZE: f32 = 0.65;
const FOOD_SIZE: f32 = 0.8;
const SPAWN_TIMING: u64 = 400;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 2000,
            height: 2000,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_resource(GameOver(false))
        .add_startup_system(setup.system())
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .init_resource::<TailPositions>()
        .add_resource(SnakeGrowth(0))
        .add_resource(SnakeMoveTimer(Timer::new(
            Duration::from_millis((SNAKE_SPEED * 1000.) as u64),
            true,
        )))
        .add_resource(FoodSpawnTimer(Timer::new(
            Duration::from_millis(SPAWN_TIMING),
            true,
        )))
        .add_system(snake_movement.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(food_spawner.system());
    }
}

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

#[derive(Default, Debug)]
struct SnakeSegment {
    next_segment: Option<Entity>,
}

struct SnakeHead {
    direction: Direction,
    next_segment: Option<Entity>,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self: &Direction) -> Direction {
        match self {
            &Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

struct SnakeMoveTimer(Timer);
struct FoodSpawnTimer(Timer);
struct SnakeGrowth(i32);
#[derive(Debug)]
struct Food;
struct GameOver(bool);
#[derive(Default)]
struct TailPositions {
    positions: HashSet<Position>,
}

fn food_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        let x = (random::<f32>() * WIDTH as f32) as i32;
        let y = (random::<f32>() * HEIGHT as f32) as i32;
        commands
            .spawn(SpriteComponents {
                material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
                // transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                sprite: Sprite::new(Vec2::default()),
                ..Default::default()
            })
            .with(Food)
            .with(Position { x, y })
            .with(Size {
                width: FOOD_SIZE,
                height: FOOD_SIZE,
            });
    }
}

fn game_over_system(
    mut commands: Commands,
    mut game_over: ResMut<GameOver>,
    mut segments: Query<(&Entity, &SnakeSegment)>,
    mut head: Query<(&Entity, &SnakeHead)>,
) {
    for (ent, _segment) in &mut segments.iter() {
        commands.despawn(*ent);
    }
    for (ent, _head) in &mut head.iter() {
        commands.despawn(*ent);
    }
}

fn snake_movement(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut game_over: ResMut<GameOver>,
    mut tail_positions: ResMut<TailPositions>,
    mut snake_growth: ResMut<SnakeGrowth>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut snake_timer: ResMut<SnakeMoveTimer>,
    mut head_positions: Query<(&mut SnakeHead, &mut Position)>,
    segments: Query<&mut SnakeSegment>,
    positions: Query<&mut Position>,
    mut food_positions: Query<(Entity, &Food, &Position)>,
    // mut food_positions_blah: Query<(&Food, &Position)>,
) {
    snake_timer.0.tick(time.delta_seconds);
    for (mut head, pos) in &mut head_positions.iter() {
        let mut intended_dir: Option<Direction> = None;
        if keyboard_input.pressed(KeyCode::Left) {
            intended_dir = Some(Direction::Left);
        }
        if keyboard_input.pressed(KeyCode::Down) {
            intended_dir = Some(Direction::Down);
        }
        if keyboard_input.pressed(KeyCode::Up) {
            intended_dir = Some(Direction::Up);
        }
        if keyboard_input.pressed(KeyCode::Right) {
            intended_dir = Some(Direction::Right);
        }
        if (intended_dir != Some(head.direction.opposite())) {
            if let Some(dir) = intended_dir {
                head.direction = dir;
            }
        }
    }
    if snake_timer.0.finished {
        let mut direction: Option<Direction> = None;
        let mut next_entity: Option<Entity> = None;
        let mut last_position: Option<Position> = None;
        let mut current_entity: Option<Entity> = None;
        for (head, mut pos) in &mut head_positions.iter() {
            if tail_positions.positions.contains(&pos) {
                game_over.0 = true;
                println!("GAME OVER");
            }
            last_position = Some(pos.clone());
            adjust_position(&mut pos, &head.direction);
            next_entity = head.next_segment;
            current_entity = head.next_segment;
            for (ent, food, food_pos) in &mut food_positions.iter() {
                if food_pos == &*pos {
                    snake_growth.0 += 1;
                    commands.despawn(ent);
                }
            }
        }
        tail_positions.positions.clear();
        while let Some(next) = next_entity {
            current_entity = Some(next);
            let mut pos = positions.get_mut::<Position>(next).unwrap();
            let segment = segments.get_mut::<SnakeSegment>(next).unwrap();
            let p = Some(pos.clone());
            pos.x = last_position.unwrap().x;
            pos.y = last_position.unwrap().y;
            tail_positions.positions.insert(*pos);
            last_position = p;
            next_entity = segment.next_segment;
        }
        if snake_growth.0 > 0 {
            snake_growth.0 -= 1;
            commands
                .spawn(SpriteComponents {
                    material: materials.add(SEGMENT_COLOR.into()),
                    sprite: Sprite::new(Vec2::new(0.0, 0.0)),
                    ..Default::default()
                })
                .with(SnakeSegment { next_segment: None })
                .with(last_position.unwrap())
                .with(Size {
                    width: SEGMENT_SIZE,
                    height: SEGMENT_SIZE,
                });
            let new_segment = commands.current_entity();
            let mut segment = segments
                .get_mut::<SnakeSegment>(current_entity.unwrap())
                .unwrap();
            segment.next_segment = new_segment;
        }
    }
}

fn adjust_position(pos: &mut Position, dir: &Direction) {
    match &dir {
        Direction::Left => {
            pos.x -= 1;
        }
        Direction::Right => {
            pos.x += 1;
        }

        Direction::Up => {
            pos.y += 1;
        }
        Direction::Down => {
            pos.y -= 1;
        }
    };
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands
        .spawn(SpriteComponents {
            material: materials.add(SEGMENT_COLOR.into()),
            // transform: Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
            sprite: Sprite::new(Vec2::new(0.0, 0.0)),
            ..Default::default()
        })
        .with(SnakeSegment { next_segment: None })
        .with(Position { x: 10, y: 8 })
        .with(Size {
            width: SEGMENT_SIZE,
            height: SEGMENT_SIZE,
        });
    let mut second_segment = commands.current_entity();
    println!("second segment: {:?}", second_segment);
    commands
        .spawn(SpriteComponents {
            material: materials.add(SEGMENT_COLOR.into()),
            // transform: Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
            sprite: Sprite::new(Vec2::new(0.0, 0.0)),
            ..Default::default()
        })
        .with(SnakeSegment {
            next_segment: second_segment,
        })
        .with(Position { x: 10, y: 9 })
        .with(Size {
            width: SEGMENT_SIZE,
            height: SEGMENT_SIZE,
        });
    let mut first_segment = commands.current_entity();
    println!("first segment: {:?}", first_segment);
    commands
        // .spawn(UiCameraComponents::default())
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
            // transform: Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
            sprite: Sprite::new(Vec2::new(0.0, 0.0)),
            ..Default::default()
        })
        .with(SnakeHead {
            direction: Direction::Up,
            next_segment: first_segment,
        })
        .with(Position { x: 10, y: 10 })
        .with(Size {
            width: 0.8,
            height: 0.8,
        });
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    for (size, mut sprite) in &mut q.iter() {
        let window = windows.get_primary().unwrap();
        sprite.size = Vec2::new(
            size.width as f32 / WIDTH as f32 * window.width as f32,
            size.height as f32 / HEIGHT as f32 * window.height as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(p: i32, bound_window: u32, bound_game: u32) -> f32 {
        return p as f32 / bound_game as f32 * bound_window as f32 - (bound_window as f32 / 2.);
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in &mut q.iter() {
        transform.set_translation(Vec3::new(
            convert(pos.x, window.width, WIDTH),
            convert(pos.y, window.height, HEIGHT),
            0.0,
        ))
    }
}

// fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
// for (size, mut sprite) in &mut q.iter() {
// let window = windows.get_primary().unwrap();
// sprite.size = Vec2::new(
// size.width as f32 / WIDTH as f32 * window.width as f32,
// size.height as f32 / HEIGHT as f32 * window.height as f32,
// );
// }
// }

fn main() {
    App::build()
        .add_plugin(HelloPlugin)
        .add_default_plugins()
        .run();
}
