#![warn(clippy::complexity)]
use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::time::Duration;

const ARENA_HEIGHT: u32 = 10;
const ARENA_WIDTH: u32 = 10;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Copy, Clone)]
struct SnakeHead {
    direction: Direction,
}

struct SnakeSegment {
    lifetime: u32,
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

struct SnakeMoveTimer(Timer);

struct GameOverEvent;

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
    });
}

fn game_setup(mut commands: Commands) {
    spawn_initial_snake(&mut commands)
}

fn spawn_initial_snake(mut commands: &mut Commands) {
    let e = spawn_segment(&mut commands, Position { x: 3, y: 2 }, 2);
    commands.insert_one(
        e,
        SnakeHead {
            direction: Direction::Up,
        },
    );
}

fn spawn_segment(commands: &mut Commands, position: Position, lifetime: u32) -> Entity {
    commands
        .spawn(SpriteComponents::default())
        .with(SnakeSegment { lifetime })
        .with(position)
        .with(Size::square(0.65));
    commands.current_entity().unwrap()
}

fn snake_movement(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut heads: Query<(Entity, &mut SnakeHead, &SnakeSegment, &Position)>,
    segment_positions: Query<With<SnakeSegment, &Position>>,
) {
    for (head_entity, mut head, head_segment, head_pos) in heads.iter_mut() {
        let dir: Direction = if keyboard_input.just_pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.just_pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.just_pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.just_pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
        if !snake_timer.0.finished {
            continue;
        }

        let new_pos = {
            let mut pos = *head_pos;
            match &head.direction {
                Direction::Left => pos.x -= 1,
                Direction::Right => pos.x += 1,
                Direction::Up => pos.y += 1,
                Direction::Down => pos.y -= 1,
            }
            pos
        };
        if new_pos.x < 0
            || new_pos.y < 0
            || new_pos.x as u32 >= ARENA_WIDTH
            || new_pos.y as u32 >= ARENA_HEIGHT
            || segment_positions.iter().any(|x| *x == new_pos)
        {
            game_over_events.send(GameOverEvent);
            return;
        }

        let e = spawn_segment(&mut commands, new_pos, head_segment.lifetime);
        commands.insert_one(e, *head);
        commands.remove_one::<SnakeHead>(head_entity);
    }
}

fn snake_decay(
    mut commands: Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut snake_segments: Query<(Entity, &mut SnakeSegment)>,
) {
    if !snake_timer.0.finished {
        return;
    }
    for (e, mut seg) in snake_segments.iter_mut() {
        if seg.lifetime == 0 {
            commands.despawn(e);
        } else {
            seg.lifetime -= 1;
        }
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    segments: Query<(Entity, &SnakeSegment)>,
    food: Query<(Entity, &Food)>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for (ent, _) in segments.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in food.iter() {
            commands.despawn(ent);
        }
        spawn_initial_snake(&mut commands);
    }
}

fn snake_eating(
    mut commands: Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
    mut head_positions: Query<With<SnakeHead, (&Position, &mut SnakeSegment)>>,
) {
    if !snake_timer.0.finished {
        return;
    }
    for (head_pos, mut head_segment) in head_positions.iter_mut() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                head_segment.lifetime += 1;
            }
        }
    }
}

fn snake_appearance(
    materials: Res<Materials>,
    mut heads: Query<With<SnakeHead, &mut Handle<ColorMaterial>>>,
    mut tails: Query<With<SnakeSegment, Without<SnakeHead, &mut Handle<ColorMaterial>>>>,
) {
    for mut h in heads.iter_mut() {
        *h = materials.head_material.clone();
    }
    for mut h in tails.iter_mut() {
        *h = materials.segment_material.clone();
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    for (size, mut sprite) in q.iter_mut() {
        let window = windows.get_primary().unwrap();
        sprite.size = Vec2::new(
            size.width as f32 / ARENA_WIDTH as f32 * window.width() as f32,
            size.height as f32 / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(p: f32, bound_window: f32, bound_game: f32) -> f32 {
        p / bound_game * bound_window - (bound_window / 2.) + (bound_window / bound_game / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(Size::square(0.8));
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 2000,
            height: 2000,
            ..Default::default()
        })
        .add_resource(SnakeMoveTimer(Timer::new(
            Duration::from_millis(150. as u64),
            true,
        )))
        .add_event::<GameOverEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(snake_timer.system())
        .add_system(snake_eating.system())
        .add_system(snake_movement.system())
        .add_system(snake_decay.system())
        .add_system(snake_appearance.system())
        .add_system(food_spawner.system())
        .add_system(game_over.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_plugins(DefaultPlugins)
        .run();
}
