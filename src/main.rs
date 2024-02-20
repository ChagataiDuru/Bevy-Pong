use bevy::{
    prelude::*,
    math::*,
    sprite::MaterialMesh2dBundle,
    math::bounding::{Aabb2d, IntersectsVolume},
};

const BALL_SPEED: f32 = 5.;
const BALL_SIZE: f32 = 5.;

#[derive(Component)]
struct Shape(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Ball;

#[derive(Bundle)]
struct BallBundle {
    ball: Ball,
    shape: Shape,
    velocity: Velocity,
    position: Position,
}

impl BallBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ball: Ball,
            shape: Shape(Vec2::new(BALL_SIZE, BALL_SIZE)),
            velocity: Velocity(Vec2::new(x, y)),
            position: Position(Vec2::new(0., 0.)),
        }
    }
}

const PADDLE_SPEED: f32 = 1.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_HEIGHT: f32 = 50.;

#[derive(Component)]
struct Paddle;

#[derive(Bundle)]
struct PaddleBundle {
    paddle: Paddle,
    shape: Shape,
    velocity: Velocity,
    position: Position,
}

impl PaddleBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            paddle: Paddle,
            shape: Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(Vec2::new(0., 0.)),
        }
    }
}

const GUTTER_HEIGHT: f32 = 20.;

#[derive(Component)]
struct Gutter;

#[derive(Bundle)]
struct GutterBundle {
    gutter: Gutter,
    shape: Shape,
    position: Position,
}

impl GutterBundle {
    fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            gutter: Gutter,
            shape: Shape(Vec2::new(width, GUTTER_HEIGHT)),
            position: Position(Vec2::new(x, y)),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (spawn_ball, spawn_camera, spawn_paddles, spawn_gutters),
        )
        .add_systems(
            Update,
            (
                move_ball,
                project_positions.after(move_ball),
                handle_collisions.after(move_ball),
            ),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty()
            .insert(Camera2dBundle::default());
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Spawning ball...");

    let mesh = Mesh::from(Circle::new(BALL_SIZE));
    let material = ColorMaterial::from(Color::rgb(1., 0., 0.));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(material);

    commands.spawn((
        BallBundle::new(1., 0.),
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            ..default()
        },
    ));
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    println!("Spawning paddles...");

    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();

        let padding = 50.;
        let right_paddle_x = window_width / 2. - padding;
        let left_paddle_x = -window_width / 2. + padding;

        let mesh = Mesh::from(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT));
        let mesh_handle = meshes.add(mesh);

        let right_paddle_material = ColorMaterial::from(Color::rgb(0., 1., 0.));
        let left_paddle_material = ColorMaterial::from(Color::rgb(0., 0., 1.));

        commands.spawn((
            PaddleBundle::new(right_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: materials.add(right_paddle_material),
                ..default()
            },
        ));

        commands.spawn((
            PaddleBundle::new(left_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: materials.add(left_paddle_material),
                ..default()
            },
        ));
    }
}

fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();

        let top_gutter_y = window_height / 2. - GUTTER_HEIGHT / 2.;
        let bottom_gutter_y = -window_height / 2. + GUTTER_HEIGHT / 2.;

        let top_gutter = GutterBundle::new(0., top_gutter_y, window_width);
        let bottom_gutter = GutterBundle::new(0., bottom_gutter_y, window_width);

        let mesh = Mesh::from(Rectangle::new(top_gutter.shape.0.x, bottom_gutter.shape.0.y));
        let material = ColorMaterial::from(Color::rgb(0., 0., 0.));

        // Sharing same mesh and material between top and bottom gutter (wow)
        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(material);

        commands.spawn((
            top_gutter,
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: material_handle.clone(),
                ..default()
            },
        ));

        commands.spawn((
            bottom_gutter,
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle.clone(),
                ..default()
            },
        ));
    }
}

fn move_ball(mut ball: Query<(&mut Position, &Velocity), With<Ball>>) {
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0
    }
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    others: Query<(&Position, &Shape), Without<Ball>>,
) {
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        for (position, shape) in &others {
            let collision = Aabb2d::new(ball_position.0.extend(0.).truncate(), ball_shape.0)
                .intersects(&Aabb2d::new(position.0.extend(0.).truncate(), shape.0));
            if collision {
                ball_velocity.0.x *= -1.;
            }
        }
    }
}

fn project_positions(
    mut positionables: Query<(&mut Transform, &Position)>
) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}