use bevy::{prelude::*, window::PrimaryWindow};
use rand::{rngs::ThreadRng, thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Flappy Bird"),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: Vec2::new(512., 512.).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup_level)
        .add_systems(Update, (update_bird, update_obstacles))
        .run();
}

const PIXEL_RATIO: f32 = 4.0;
const GRAVITY: f32 = 2000.;
const BIRD_ROTATION_RATIO: f32 = 500.;
const BIRD_FLAP_VELOCITY: f32 = 500.;
const OBSTACLE_AMOUNT: i32 = 10;
const OBSTACLE_SPEED: f32 = 150.0;
const OBSTACLE_WIDTH: f32 = 32.0;
const OBSTACLE_RANDOM_VERTICAL_OFFSET: f32 = 30.0;
const OBSTACLE_GAP_SIZE: f32 = 15.0;
const OBSTACLE_SPACING: f32 = 60.0;
const OBSTACLE_IMAGE_HEIGHT: f32 = 144.0;

#[derive(Component)]
pub struct Bird {
    pub velocity: f32,
}
#[derive(Resource)]
pub struct GameManager {
    pub pipe_image: Handle<Image>,
    pub window_dimensions: Vec2,
}
pub fn setup_level(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();
    let pipe_image = asset_server.load("pipe.png");

    commands.insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.8)));
    commands.insert_resource(GameManager {
        pipe_image: pipe_image.clone(),
        window_dimensions: Vec2::new(window.width(), window.height()),
    });
    commands.spawn(Camera2d {
        ..Default::default()
    });
    commands.spawn((
        Sprite {
            image: asset_server.load("Bird.png"),
            ..Default::default()
        },
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
        Bird { velocity: 0. },
    ));
    let mut rand = thread_rng();
    spawn_obstacles(&mut commands, &mut rand, window.width(), &pipe_image);
}
pub fn spawn_obstacles(
    mut commands: &mut Commands,
    mut rand: &mut ThreadRng,
    window_width: f32,
    pipe_image: &Handle<Image>,
) {
    for i in 0..OBSTACLE_AMOUNT {
        let y_offset = generate_random_pipe_offset(&mut rand);
        let x_position = window_width / 2. + OBSTACLE_SPACING * PIXEL_RATIO * i as f32;
        spawn_obstacle(
            Vec3::X * x_position + Vec3::Y * (get_pipe_centered_position() + y_offset),
            1.,
            &mut commands,
            pipe_image,
        );
        spawn_obstacle(
            Vec3::X * x_position + Vec3::Y * (-get_pipe_centered_position() + y_offset),
            -1.,
            &mut commands,
            pipe_image,
        );
    }
}
pub fn generate_random_pipe_offset(rand: &mut ThreadRng) -> f32 {
    return rand.gen_range(-OBSTACLE_RANDOM_VERTICAL_OFFSET..OBSTACLE_RANDOM_VERTICAL_OFFSET)
        * PIXEL_RATIO;
}
pub fn get_pipe_centered_position() -> f32 {
    return (OBSTACLE_IMAGE_HEIGHT / 2. + OBSTACLE_GAP_SIZE) * PIXEL_RATIO;
}
pub fn spawn_obstacle(
    translation: Vec3,
    pipe_direction: f32,
    commands: &mut Commands,
    pipe_image: &Handle<Image>,
) {
    commands.spawn((
        Sprite {
            image: pipe_image.clone(),
            ..Default::default()
        },
        Transform::from_translation(translation).with_scale(Vec3::new(
            PIXEL_RATIO,
            PIXEL_RATIO * -pipe_direction,
            PIXEL_RATIO,
        )),
        Obstacle { pipe_direction },
    ));
}
pub fn update_bird(
    mut commands: Commands,
    mut bird_query: Query<(&mut Bird, &mut Transform), Without<Obstacle>>,
    mut obstacle_query: Query<(&Transform, Entity), With<Obstacle>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game_manager: Res<GameManager>,
) {
    if let Ok((mut bird, mut transform)) = bird_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Space) {
            bird.velocity = BIRD_FLAP_VELOCITY;
        }
        bird.velocity -= time.delta_secs() * GRAVITY;
        transform.translation.y += bird.velocity * time.delta_secs();

        transform.rotation = Quat::from_axis_angle(
            Vec3::Z,
            f32::clamp(bird.velocity / BIRD_ROTATION_RATIO, -90., 90.),
        );

        let mut bird_dead = false;
        if transform.translation.y <= -game_manager.window_dimensions.y / 2. {
            bird_dead = true;
        } else {
            for (pipe_transform, _entity) in obstacle_query.iter() {
                if (pipe_transform.translation.y - transform.translation.y).abs()
                    < OBSTACLE_IMAGE_HEIGHT * PIXEL_RATIO / 2.
                    && (pipe_transform.translation.x - transform.translation.x).abs()
                        < OBSTACLE_WIDTH * PIXEL_RATIO / 2.
                {
                    bird_dead = true;
                    break;
                }
            }
        }
        if bird_dead {
            transform.translation = Vec3::ZERO;
            bird.velocity = 0.;
            for (_pipe_transform, entity) in obstacle_query.iter_mut() {
                commands.entity(entity).despawn();
            }
            let mut rand = thread_rng();
            spawn_obstacles(
                &mut commands,
                &mut rand,
                game_manager.window_dimensions.x,
                &game_manager.pipe_image,
            );
        }
    }
}

#[derive(Component)]
pub struct Obstacle {
    pipe_direction: f32,
}
pub fn update_obstacles(
    time: Res<Time>,
    game_manager: Res<GameManager>,
    mut obstacle_query: Query<(&mut Obstacle, &mut Transform)>,
) {
    let mut rand = thread_rng();
    let y_offset = generate_random_pipe_offset(&mut rand);
    for (obstacle, mut transform) in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * OBSTACLE_SPEED;

        if transform.translation.x + OBSTACLE_WIDTH * PIXEL_RATIO / 2.
            < -game_manager.window_dimensions.x / 2.
        {
            transform.translation.x += OBSTACLE_AMOUNT as f32 * OBSTACLE_SPACING * PIXEL_RATIO;
            transform.translation.y =
                get_pipe_centered_position() * obstacle.pipe_direction + y_offset;
        }
    }
}
