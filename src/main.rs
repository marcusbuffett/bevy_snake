#![warn(clippy::complexity, clippy::perf, clippy::nursery)]
use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::time::Duration;

const ARENA_HEIGHT: u32 = 10;
const ARENA_WIDTH: u32 = 10;

struct HeadMaterial(Handle<ColorMaterial>);
struct SegmentMaterial(Handle<ColorMaterial>);
struct SnakeHead {
    direction: Direction,
    next_segment: Entity,
}
#[derive(Default)]
struct SnakeSegment {
    next_segment: Option<Entity>,
}
struct SnakeMoveTimer(Timer);

struct FoodSpawnTimer(Timer);
struct FoodMaterial(Handle<ColorMaterial>);
struct Food;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

struct GameOverEvent;
struct GrowthEvent;

#[derive(PartialEq, Copy, Clone, Debug)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
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
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(HeadMaterial(
        materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
    ));
    commands.insert_resource(SegmentMaterial(
        materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
    ));
    commands.insert_resource(FoodMaterial(
        materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
    ));
}

fn game_setup(
    commands: Commands,
    head_material: Res<HeadMaterial>,
    segment_material: Res<SegmentMaterial>,
) {
    spawn_initial_snake(commands, &head_material.0, &segment_material.0);
}

fn spawn_segment(commands: &mut Commands, material: &Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(SpriteComponents {
            material: material.clone(),
            ..SpriteComponents::default()
        })
        .with(SnakeSegment { next_segment: None })
        .with(position)
        .with(Size::square(0.65));
}

fn spawn_initial_snake(
    mut commands: Commands,
    head_material: &Handle<ColorMaterial>,
    segment_material: &Handle<ColorMaterial>,
) {
    spawn_segment(&mut commands, segment_material, Position { x: 4, y: 0 });
    let first_segment = commands.current_entity().unwrap();
    commands
        .spawn(SpriteComponents {
            material: head_material.clone(),
            ..SpriteComponents::default()
        })
        .with(SnakeHead {
            direction: Direction::Up,
            next_segment: first_segment,
        })
        .with(Position { x: 4, y: 1 })
        .with(Size::square(0.8));
}

fn snake_movement(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut snake_timer: ResMut<SnakeMoveTimer>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    segment_material: Res<SegmentMaterial>,
    mut segments: Query<&mut SnakeSegment>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    snake_timer.0.tick(time.delta_seconds);
    for (head_entity, mut head) in heads.iter_mut() {
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
        if snake_timer.0.finished {
            let mut last_position = *head_pos;
            match &head.direction {
                Direction::Left => {
                    head_pos.x -= 1;
                }
                Direction::Right => {
                    head_pos.x += 1;
                }

                Direction::Up => {
                    head_pos.y += 1;
                }
                Direction::Down => {
                    head_pos.y -= 1;
                }
            };
            if head_pos.x < 0
                || head_pos.y < 0
                || head_pos.x as u32 > ARENA_WIDTH
                || head_pos.y as u32 > ARENA_HEIGHT
            {
                game_over_events.send(GameOverEvent);
            }
            let head_pos = *head_pos;
            let mut segment_entity = head.next_segment;
            loop {
                let segment = segments.get_mut(segment_entity).unwrap();
                let mut segment_position = positions.get_mut(segment_entity).unwrap();
                if head_pos == *segment_position {
                    game_over_events.send(GameOverEvent);
                }
                last_tail_position.0 = Some(*segment_position);
                std::mem::swap(&mut last_position, &mut *segment_position);
                if let Some(next_entity) = segment.next_segment {
                    segment_entity = next_entity;
                } else {
                    break;
                }
            }
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
    head_positions: Query<With<SnakeHead, &Position>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
            }
        }
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

fn snake_growth(
    mut commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    mut last_segment: Local<Option<Entity>>,
    segment_material: Res<SegmentMaterial>,
    mut segments: Query<&mut SnakeSegment>,
    mut heads: Query<&SnakeHead>,
    mut positions: Query<&SnakeHead>,
) {
    if let Some(head) = heads.iter().next() {
        if growth_reader.iter(&growth_events).next().is_some() {
            let mut last_segment = head.next_segment;
            loop {
                let segment = segments.get_mut(last_segment).unwrap();
                if let Some(next_entity) = segment.next_segment {
                    last_segment = next_entity;
                } else {
                    break;
                }
            }
            spawn_segment(
                &mut commands,
                &segment_material.0,
                last_tail_position.0.unwrap(),
            );
            let new_segment = commands.current_entity();
            let mut segment = segments.get_mut(last_segment).unwrap();
            segment.next_segment = new_segment;
        }
    }
}

fn game_over_system(
    mut commands: Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    segment_material: Res<SegmentMaterial>,
    head_material: Res<HeadMaterial>,
    segments: Query<(Entity, &SnakeSegment)>,
    food: Query<(Entity, &Food)>,
    heads: Query<(Entity, &SnakeHead)>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for (ent, _) in segments.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in food.iter() {
            commands.despawn(ent);
        }
        for (ent, _) in heads.iter() {
            commands.despawn(ent);
        }
        spawn_initial_snake(commands, &head_material.0, &segment_material.0);
    }
}

fn food_spawner(
    mut commands: Commands,
    food_material: Res<FoodMaterial>,
    time: Res<Time>,
    mut timer: ResMut<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        commands
            .spawn(SpriteComponents {
                material: food_material.0.clone(),
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
        .add_event::<GrowthEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        .add_system(snake_eating.system())
        .add_system(snake_movement.system())
        .add_system(snake_growth.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_resource(FoodSpawnTimer(Timer::new(
            Duration::from_millis(1000),
            true,
        )))
        .add_resource(LastTailPosition::default())
        .add_system(food_spawner.system())
        .add_system(game_over_system.system())
        .add_plugins(DefaultPlugins)
        .run();
}
