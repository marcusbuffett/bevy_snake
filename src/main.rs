use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::collections::hash_set::HashSet;
use std::time::Duration;

// -- Constants --
const ARENA_WIDTH: u32 = 40;
const ARENA_HEIGHT: u32 = 40;
const SNAKE_SPEED: f64 = 1. / 15.;
const SPAWN_TIMING: u64 = 400;

// -- Resources --
struct SnakeMoveTimer(Timer);

struct FoodSpawnTimer(Timer);

#[derive(Copy, Clone)]
struct SegmentMaterial(Handle<ColorMaterial>);

struct HeadMaterial(Handle<ColorMaterial>);

struct FoodMaterial(Handle<ColorMaterial>);

struct SnakeGrowth(i32);

// -- Components --
#[derive(Debug)]
struct Food;

struct GameOver(bool);

#[derive(Default)]
struct TailPositions {
    positions: HashSet<Position>,
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
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

// -- Startup Systems --

fn materials_setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(SegmentMaterial(
        materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
    ));
    commands.insert_resource(FoodMaterial(
        materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
    ));
    commands.insert_resource(HeadMaterial(
        materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
    ));
}

fn game_setup(
    mut commands: Commands,
    segment_material: Res<SegmentMaterial>,
    head_material: Res<HeadMaterial>,
) {
    commands.spawn(Camera2dComponents::default());
    spawn_initial_snake(commands, head_material.0, segment_material.0);
}

// -- Systems --

fn food_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        let x = (random::<f32>() * ARENA_WIDTH as f32) as i32;
        let y = (random::<f32>() * ARENA_HEIGHT as f32) as i32;
        commands
            .spawn(SpriteComponents {
                material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
                // transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                sprite: Sprite::new(Vec2::default()),
                ..Default::default()
            })
            .with(Food)
            .with(Position { x, y })
            .with(Size::square(0.8));
    }
}

fn game_over_system(
    mut commands: Commands,
    segment_material: Res<SegmentMaterial>,
    head_material: Res<HeadMaterial>,
    mut game_over: ResMut<GameOver>,
    mut segments: Query<(Entity, &SnakeSegment)>,
    mut food: Query<(Entity, &Food)>,
    mut heads: Query<(Entity, &SnakeHead)>,
) {
    if game_over.0 {
        for (ent, _segment) in &mut segments.iter() {
            commands.despawn(ent);
        }
        for (ent, _food) in &mut food.iter() {
            commands.despawn(ent);
        }
        for (ent, _head) in &mut heads.iter() {
            commands.despawn(ent);
        }
        game_over.0 = false;
        spawn_initial_snake(commands, head_material.0, segment_material.0);
    }
}

fn snake_movement(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    segment_material: Res<SegmentMaterial>,
    mut game_over: ResMut<GameOver>,
    mut tail_positions: ResMut<TailPositions>,
    mut snake_growth: ResMut<SnakeGrowth>,
    mut snake_timer: ResMut<SnakeMoveTimer>,
    mut head_positions: Query<(&mut SnakeHead, &mut Position)>,
    segments: Query<&mut SnakeSegment>,
    positions: Query<&mut Position>,
    mut food_positions: Query<(Entity, &Food, &Position)>,
) {
    snake_timer.0.tick(time.delta_seconds);
    for (mut head, _pos) in &mut head_positions.iter() {
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
        if intended_dir != Some(head.direction.opposite()) {
            if let Some(dir) = intended_dir {
                head.direction = dir;
            }
        }
    }
    if snake_timer.0.finished {
        let mut next_entity: Option<Entity> = None;
        let mut last_position: Option<Position> = None;
        let mut current_entity: Option<Entity> = None;
        for (head, mut pos) in &mut head_positions.iter() {
            if tail_positions.positions.contains(&pos) {
                game_over.0 = true;
            }
            last_position = Some(*pos);
            adjust_position(&mut pos, &head.direction);
            next_entity = head.next_segment;
            current_entity = head.next_segment;
            for (ent, _food, food_pos) in &mut food_positions.iter() {
                if food_pos == &*pos {
                    snake_growth.0 += 10;
                    commands.despawn(ent);
                }
            }
        }
        tail_positions.positions.clear();
        while let Some(next) = next_entity {
            current_entity = Some(next);
            let mut pos = positions.get_mut::<Position>(next).unwrap();
            let segment = segments.get_mut::<SnakeSegment>(next).unwrap();
            let p = Some(*pos);
            pos.x = last_position.unwrap().x;
            pos.y = last_position.unwrap().y;
            tail_positions.positions.insert(*pos);
            last_position = p;
            next_entity = segment.next_segment;
        }
        if snake_growth.0 > 0 {
            snake_growth.0 -= 1;
            spawn_segment(
                &mut commands,
                segment_material.0,
                last_position.unwrap(),
                None,
            );
            let new_segment = commands.current_entity();
            let mut segment = segments
                .get_mut::<SnakeSegment>(current_entity.unwrap())
                .unwrap();
            segment.next_segment = new_segment;
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    for (size, mut sprite) in &mut q.iter() {
        let window = windows.get_primary().unwrap();
        sprite.size = Vec2::new(
            size.width as f32 / ARENA_WIDTH as f32 * window.width as f32,
            size.height as f32 / ARENA_HEIGHT as f32 * window.height as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(p: f32, bound_window: f32, bound_game: f32) -> f32 {
        p / bound_game * bound_window - (bound_window / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in &mut q.iter() {
        transform.set_translation(Vec3::new(
            convert(pos.x as f32, window.width as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height as f32, ARENA_HEIGHT as f32),
            0.0,
        ))
    }
}

// -- Helpers --

fn spawn_segment(
    commands: &mut Commands,
    material: Handle<ColorMaterial>,
    position: Position,
    next_segment: Option<Entity>,
) {
    commands
        .spawn(SpriteComponents {
            material,
            sprite: Sprite::new(Vec2::new(0.0, 0.0)),
            ..Default::default()
        })
        .with(SnakeSegment { next_segment })
        .with(position)
        .with(Size::square(0.65));
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

fn spawn_initial_snake(
    mut commands: Commands,
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
) {
    spawn_segment(
        &mut commands,
        segment_material,
        Position { x: 10, y: 8 },
        None,
    );
    let second_segment = commands.current_entity();
    spawn_segment(
        &mut commands,
        segment_material,
        Position { x: 10, y: 9 },
        second_segment,
    );
    let first_segment = commands.current_entity();
    commands
        // .spawn(UiCameraComponents::default())
        .spawn(SpriteComponents {
            material: head_material,
            // transform: Transform::from_translation(Vec3::new(50.0, 50.0, 0.0)),
            sprite: Sprite::new(Vec2::new(0.0, 0.0)),
            ..Default::default()
        })
        .with(SnakeHead {
            direction: Direction::Up,
            next_segment: first_segment,
        })
        .with(Position { x: 10, y: 10 })
        .with(Size::square(0.8));
}

fn main() {
    App::build()
        // Background color
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 2000,
            height: 2000,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_startup_system(materials_setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", game_setup.system())
        // -- Resources --
        .add_resource(GameOver(false))
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
        // -- Game systems -
        .add_system(snake_movement.system())
        .add_system(game_over_system.system())
        .add_system(food_spawner.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        // Default stuff
        .add_default_plugins()
        .run();
}
