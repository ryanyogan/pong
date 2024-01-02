use bevy::{
    prelude::*,
    sprite::{
        collide_aabb::{collide, Collision},
        MaterialMesh2dBundle,
    },
};

const BALL_SIZE: f32 = 5.;
const BALL_SPEED: f32 = 5.;

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
    position: Position,
    shape: Shape,
    velocity: Velocity,
}

impl PaddleBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            paddle: Paddle,
            position: Position(Vec2::new(x, y)),
            shape: Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
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

        let mesh = Mesh::from(shape::Quad::new(top_gutter.shape.0));
        let material = ColorMaterial::from(Color::rgb(0., 0., 0.));

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

enum Scorer {
    Ai,
    Player,
}

#[derive(Event)]
struct Scored(Scorer);

#[derive(Resource, Default)]
struct Score {
    player: u32,
    ai: u32,
}

fn detect_scoring(
    mut ball: Query<&mut Position, With<Ball>>,
    window: Query<&Window>,
    mut events: EventWriter<Scored>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();

        if let Ok(ball) = ball.get_single_mut() {
            if ball.0.x > window_width / 2. {
                events.send(Scored(Scorer::Ai));
            } else if ball.0.x < -window_width / 2. {
                events.send(Scored(Scorer::Player));
            }
        }
    }
}

fn reset_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<Scored>,
) {
    for event in events.iter() {
        if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
            match event.0 {
                Scorer::Ai => {
                    position.0 = Vec2::new(0., 0.);
                    velocity.0 = Vec2::new(-1., 1.);
                }
                Scorer::Player => {
                    position.0 = Vec2::new(0., 0.);
                    velocity.0 = Vec2::new(1., 1.);
                }
            }
        }
    }
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
    for event in events.iter() {
        match event.0 {
            Scorer::Ai => score.ai += 1,
            Scorer::Player => score.player += 1,
        }
    }

    println!("Score: {} - {}", score.player, score.ai);
}

#[derive(Component)]
struct Ai;

#[derive(Component)]
struct Player;

fn handle_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut paddle: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut velocity) = paddle.get_single_mut() {
        if keyboard_input.pressed(KeyCode::Up) {
            velocity.0.y = 1.;
        } else if keyboard_input.pressed(KeyCode::Down) {
            velocity.0.y = -1.;
        } else {
            velocity.0.y = 0.;
        }
    }
}

fn move_paddles(
    mut paddle: Query<(&mut Position, &Velocity), With<Paddle>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();

        for (mut position, velocity) in &mut paddle {
            let new_position = position.0 + velocity.0 * PADDLE_SPEED;
            if new_position.y.abs() < window_height / 2. - GUTTER_HEIGHT - PADDLE_HEIGHT / 2. {
                position.0 = new_position;
            }
        }
    }
}

fn move_ai(
    mut ai: Query<(&mut Velocity, &Position), With<Ai>>,
    ball: Query<&Position, With<Ball>>,
) {
    if let Ok((mut velocity, position)) = ai.get_single_mut() {
        if let Ok(ball_position) = ball.get_single() {
            let a_to_b = ball_position.0 - position.0;
            velocity.0.y = a_to_b.y.signum();
        }
    }
}

#[derive(Component)]
struct PlayerScoreboard;

#[derive(Component)]
struct AiScoreboard;

fn update_scoreboard(
    mut player_score: Query<&mut Text, With<PlayerScoreboard>>,
    mut ai_score: Query<&mut Text, (With<AiScoreboard>, Without<PlayerScoreboard>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        if let Ok(mut player_score) = player_score.get_single_mut() {
            player_score.sections[0].value = score.player.to_string();
        }

        if let Ok(mut ai_score) = ai_score.get_single_mut() {
            ai_score.sections[0].value = score.ai.to_string();
        }
    }
}

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((
        PlayerScoreboard,
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "0",
            TextStyle {
                font_size: 72.0,
                color: Color::WHITE,
                ..default()
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
    ));

    commands.spawn((
        AiScoreboard,
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 72.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 0.75,
        })
        .add_event::<Scored>()
        .add_systems(
            Startup,
            (
                spawn_ball,
                spawn_camera,
                spawn_paddles,
                spawn_gutters,
                spawn_scoreboard,
            ),
        )
        .add_systems(
            Update,
            (
                move_ball,
                handle_player_input,
                detect_scoring,
                move_ai,
                reset_ball.after(detect_scoring),
                update_score.after(detect_scoring),
                update_scoreboard.after(update_score),
                move_paddles.after(handle_player_input),
                project_positions.after(move_ball),
                handle_collision.after(move_ball),
            ),
        )
        .run();
}

fn handle_collision(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    other_things: Query<(&Position, &Shape), Without<Ball>>,
) {
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        for (position, shape) in &other_things {
            if let Some(collision) = collide(
                ball_position.0.extend(0.),
                ball_shape.0,
                position.0.extend(0.),
                shape.0,
            ) {
                match collision {
                    Collision::Left => {
                        ball_velocity.0.x *= -1.;
                    }
                    Collision::Right => {
                        ball_velocity.0.x *= 1.;
                    }
                    Collision::Top => {
                        ball_velocity.0.y *= -1.;
                    }
                    Collision::Bottom => {
                        ball_velocity.0.y *= 1.;
                    }
                    Collision::Inside => {}
                }
            }
        }
    }
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let padding = 50.;
        let right_paddle_x = window_width / 2. - padding;
        let left_paddle_x = -window_width / 2. + padding;

        let mesh = Mesh::from(shape::Quad::new(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)));
        let mesh_handle = meshes.add(mesh);

        let right_paddle_material = ColorMaterial::from(Color::rgb(0., 1., 0.));
        let left_paddle_material = ColorMaterial::from(Color::rgb(0., 0., 1.));

        commands.spawn((
            Player,
            PaddleBundle::new(right_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: materials.add(right_paddle_material),
                ..default()
            },
        ));

        commands.spawn((
            Ai,
            PaddleBundle::new(left_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: materials.add(left_paddle_material),
                ..default()
            },
        ));
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = Mesh::from(shape::Circle::new(BALL_SIZE));
    let material = ColorMaterial::from(Color::rgb(255., 255., 255.));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(material);

    commands.spawn((
        BallBundle::new(1., 1.1725),
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            ..default()
        },
    ));
}

fn move_ball(mut ball: Query<(&mut Position, &Velocity), With<Ball>>) {
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0;
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}

fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}
