use bevy::{prelude::*, window::PrimaryWindow};
use rand::{rngs::ThreadRng, thread_rng, Rng};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_level)
        .add_systems(Update, (update_bird, update_obstacles))
        .run();
}
const GRAVITY: f32 = 2000.;
const OBSTACLE_SPEED: f32 = 500.0;
const OBSTACLE_WIDTH: f32 = 32.0;
const OBSTACLE_RANDOM_VERTICAL_OFFSET: f32 = 50.0;
const GAP_SIZE: f32 = 20.0;
const OBSTACLE_SPACING: f32 = 400.0;
const OBSTACLE_IMAGE_HEIGHT: f32 = 144.0;
const PIXEL_RATIO: f32 = 4.0;
const OBSTACLE_AMOUNT : i32 = 5;
#[derive(Component)]
pub struct Bird {
    pub velocity: f32,
    pub flap_force: f32,
}
pub fn setup_level(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::IDENTITY,
        ..Default::default()
    });
    commands.spawn((
        SpriteBundle {
            transform: Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
            texture: asset_server.load("Bird.png"),
            ..Default::default()
        },
        Bird {
            velocity: 0.,
            flap_force: 500.,
        },
    ));
    let mut rand = thread_rng();
    let y_offset = generate_random_pipe_offset(&mut rand);
    for i in 0..OBSTACLE_AMOUNT {
        spawn_obstacle(
            Vec3::new(
                OBSTACLE_SPACING * i as f32,
                get_pipe_centered_position(1.)+y_offset,
                0.,
            ),
            1.,
            &mut commands,
            &asset_server,
        );
        spawn_obstacle(
            Vec3::new(
                OBSTACLE_SPACING * i as f32,
                get_pipe_centered_position(-1.)+y_offset,
                0.,
            ),
            -1.,
            &mut commands,
            &asset_server,
        );
    }
}
pub fn generate_random_pipe_offset(rand : &mut ThreadRng) -> f32{
    return rand.gen_range(-OBSTACLE_RANDOM_VERTICAL_OFFSET..OBSTACLE_RANDOM_VERTICAL_OFFSET)*PIXEL_RATIO;
}
pub fn get_pipe_centered_position(vertical: f32) -> f32 {
    return (((OBSTACLE_IMAGE_HEIGHT / 2.) + GAP_SIZE) * vertical) * PIXEL_RATIO;
}
pub fn spawn_obstacle(
    translation: Vec3,
    vertical: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(translation).with_scale(Vec3::new(
                PIXEL_RATIO,
                PIXEL_RATIO * -vertical,
                PIXEL_RATIO,
            )),
            texture: asset_server.load("pipe.png"),
            ..Default::default()
        },
        Obstacle { vertical: vertical },
    ));
}
pub fn update_bird(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut bird_query: Query<(&mut Bird, &mut Transform)>,
) {
    if let Ok((mut bird, mut transform)) = bird_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Space) {
            bird.velocity = bird.flap_force;
        }
        bird.velocity -= time.delta_seconds() * GRAVITY;
        transform.translation.y += bird.velocity * time.delta_seconds();
        
        transform.rotation =
            Quat::from_axis_angle(Vec3::Z, f32::clamp(bird.velocity / 500., -90., 90.));

        //collision
    }
}

#[derive(Component)]
pub struct Obstacle {
    vertical: f32,
}
pub fn update_obstacles(
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut obstacle_query: Query<(&mut Obstacle, &mut Transform)>,
) {
    let window = window_query.get_single().unwrap();
    let mut rand = thread_rng();
    let y_offset = generate_random_pipe_offset(&mut rand);
    for (obstacle, mut transform) in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_seconds() * OBSTACLE_SPEED;

        if transform.translation.x + OBSTACLE_WIDTH / 2. < -window.width() / 2. {
            transform.translation.x += OBSTACLE_AMOUNT as f32 * OBSTACLE_SPACING;
            transform.translation.y = get_pipe_centered_position(obstacle.vertical)+y_offset;
        }
    }
}
