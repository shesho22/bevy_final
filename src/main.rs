use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin, AudioSource};
use rand::Rng;

// ===================================
// ===        CONSTANTES GLOBALES  ===
// ===================================
const PLAYER_START_POS: Vec3 = Vec3::new(-300.0, -60.0, 0.0);
const FLOOR_Y: f32 = -100.0;
const PLAYER_HEIGHT: f32 = 50.0;

const OBSTACLE_SPACING: f32 = 220.0;
const OBSTACLE_START_X: f32 = 400.0;
const OBSTACLE_MIN_X: f32 = -550.0;
const OBSTACLE_RESET_X: f32 = 550.0;
const OBSTACLE_SPEED: f32 = 220.0;

// ===================================
// ===          COMPONENTES         ===
// ===================================
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2); // Velocidad Y (Salto y gravedad)

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Airborne(bool);

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct FrameIndex(usize);

// ===================================
// ===           ESTADOS           ===
// ===================================
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default] Menu,
    Playing,
    GameOver,
}

// ===================================
// ===           RECURSOS          ===
// ===================================
#[derive(Resource, Default)]
struct AudioHandles {
    jump: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
}

#[derive(Resource, Default)]
struct Score {
    value: f32,
}

#[derive(Resource)]
struct PlayerFrames {
    frames: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct ObstacleGroundFrames {
    frames: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct ObstacleAirFrames {
    frames: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct ObstacleTextures {
    ground: Handle<Image>,
    air: Handle<Image>,
}

// ===================================
// ===      PLUGIN: JUGADOR        ===
// ===================================
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
           .add_systems(Update, start_game.run_if(in_state(GameState::Menu)))
           .add_systems(Update, (
                move_player,
                animate_player_frames,
                update_score,
           ).run_if(in_state(GameState::Playing)));
    }
}

// ----------------------------------
// SETUP GENERAL
// ----------------------------------
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut audio_handles: ResMut<AudioHandles>,
) {
    setup_camera_and_background(&mut commands, &asset_server);
    setup_audio(&mut audio_handles, &asset_server);
    setup_frame_resources(&mut commands, &asset_server);
    setup_floor(&mut commands, &asset_server);
    setup_player(&mut commands, &asset_server);
    setup_obstacles(&mut commands, &asset_server);
    setup_score_text(&mut commands);
}

fn setup_camera_and_background(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(0.8))));
    commands.spawn((
        Sprite::from_image(asset_server.load("backgrounds/background.png")),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
    ));
}

fn setup_audio(audio: &mut AudioHandles, asset_server: &AssetServer) {
    audio.jump = asset_server.load("sounds/jump.ogg");
    audio.game_over = asset_server.load("sounds/game-over.ogg");
}

fn setup_frame_resources(commands: &mut Commands, asset_server: &AssetServer) {
    // Player frames
    let frames = vec![
        asset_server.load("sprites/character1.png"),
        asset_server.load("sprites/character2.png"),
        asset_server.load("sprites/character3.png"),
        asset_server.load("sprites/character4.png"),
    ];
    commands.insert_resource(PlayerFrames { frames: frames.clone() });

    // Obstacle frames
    let ground_frames = vec![
        asset_server.load("sprites/obstacle-ground1.png"),
        asset_server.load("sprites/obstacle-ground2.png"),
        asset_server.load("sprites/obstacle-ground3.png"),
    ];

    let air_frames = vec![
        asset_server.load("sprites/obstacle-air1.png"),
        asset_server.load("sprites/obstacle-air2.png"),
        asset_server.load("sprites/obstacle-air3.png"),
    ];

    commands.insert_resource(ObstacleGroundFrames { frames: ground_frames.clone() });
    commands.insert_resource(ObstacleAirFrames { frames: air_frames.clone() });
    commands.insert_resource(ObstacleTextures {
        ground: ground_frames[0].clone(),
        air: air_frames[0].clone(),
    });
}

fn setup_floor(commands: &mut Commands, asset_server: &AssetServer) {
    let floor_texture = asset_server.load("sprites/floor.png");
    let num_tiles = 20;
    let tile_width = 64.0;
    let y_offset = -40.0;

    for i in 0..num_tiles {
        let x = (i as f32 * tile_width) - (num_tiles as f32 * tile_width / 2.0);
        commands.spawn((
            Sprite::from_image(floor_texture.clone()),
            Transform {
                translation: Vec3::new(x, FLOOR_Y + y_offset, 0.0),
                scale: Vec3::splat(1.0),
                ..default()
            },
            Floor,
        ));
    }
}

fn setup_player(commands: &mut Commands,asset_server: &AssetServer) {
    let character = asset_server.load("sprites/character4.png");
    commands.spawn((
        Sprite::from_image(character),
        Transform::from_translation(PLAYER_START_POS).with_scale(Vec3::splat(0.5)),
        Player,
        Velocity(Vec2::ZERO),
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        FrameIndex(0),
    ));
}

fn setup_obstacles(commands: &mut Commands, asset_server: &AssetServer) {
    let ground = asset_server.load("sprites/obstacle-ground1.png");
    let air = asset_server.load("sprites/obstacle-air1.png");
    let mut rng = rand::thread_rng();

    for i in 0..5 {
        let x = OBSTACLE_START_X + i as f32 * OBSTACLE_SPACING;
        let is_airborne = rng.gen_bool(0.5);

        let (y, texture) = if is_airborne {
            (FLOOR_Y + 90.0, air.clone())
        } else {
            (FLOOR_Y + 10.0, ground.clone())
        };

        commands.spawn((
            Sprite::from_image(texture),
            Transform::from_translation(Vec3::new(x, y, 0.0))
                .with_scale(Vec3::splat(0.7)),
            Obstacle,
            Airborne(is_airborne),
            AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
            FrameIndex(0),
        ));
    }
}

fn setup_score_text(commands: &mut Commands) {
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont { font_size: 30.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(-350.0, 200.0, 1.0)),
        ScoreText,
    ));
}

fn start_game(
    input: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if input.any_pressed([KeyCode::ArrowUp, KeyCode::Space, KeyCode::Enter]) {
        next.set(GameState::Playing);
    }
}

// ----------------------------------
// SISTEMAS DEL JUGADOR
// ----------------------------------
fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut tf, mut vel)) = query.single_mut() {
        let delta = time.delta().as_secs_f32();
        let gravity = if input.pressed(KeyCode::ArrowDown) { -2000.0 } else { -500.0 };
        let jump_speed = 350.0;

        let on_floor = tf.translation.y <= FLOOR_Y + PLAYER_HEIGHT / 2.0 + 0.1;

        if input.just_pressed(KeyCode::ArrowUp) && on_floor {
            vel.0.y = jump_speed;
            audio.play(audio_handles.jump.clone());
        }

        vel.0.y += gravity * delta;

        tf.translation.y = (tf.translation.y + vel.0.y * delta)
            .max(FLOOR_Y + PLAYER_HEIGHT / 2.0);
    }
}

fn animate_player_frames(
    time: Res<Time>,
    frames: Res<PlayerFrames>,
    mut query: Query<(&mut AnimationTimer, &mut FrameIndex, &mut Sprite), With<Player>>,
) {
    for (mut timer, mut index, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            index.0 = (index.0 + 1) % frames.frames.len();
            sprite.image = frames.frames[index.0].clone();
        }
    }
}

fn update_score(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut query: Query<&mut Text2d, With<ScoreText>>,
) {
    score.value += time.delta().as_secs_f32() * 5.0;

    if let Ok(mut text) = query.single_mut() {
        text.0 = format!("Score: {}", score.value.floor() as i32);
    }
}

// ===================================
// ===     PLUGIN: OBSTACULOS      ===
// ===================================
pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_obstacles,
            animate_obstacle_frames,
            check_collision,
        ).run_if(in_state(GameState::Playing)));
    }
}

// ----------------------------------
// SISTEMAS DE OBSTÁCULOS
// ----------------------------------
fn move_obstacles(
    time: Res<Time>,
    textures: Res<ObstacleTextures>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut Airborne), With<Obstacle>>,
) {
    let delta = time.delta().as_secs_f32();
    let mut rng = rand::thread_rng();

    for (mut tf, mut sprite, mut airborne_state) in query.iter_mut() {
        tf.translation.x -= OBSTACLE_SPEED * delta;

        if tf.translation.x < OBSTACLE_MIN_X {
            airborne_state.0 = rng.gen_bool(0.5);

            tf.translation.x = OBSTACLE_RESET_X;
            tf.translation.y = if airborne_state.0 { FLOOR_Y + 90.0 } else { FLOOR_Y + 10.0 };

            sprite.image = if airborne_state.0 {
                textures.air.clone()
            } else {
                textures.ground.clone()
            };
        }
    }
}

fn animate_obstacle_frames(
    time: Res<Time>,
    ground_frames: Res<ObstacleGroundFrames>,
    air_frames: Res<ObstacleAirFrames>,
    mut query: Query<(&mut AnimationTimer, &mut FrameIndex, &mut Sprite, &Airborne), With<Obstacle>>,
) {
    for (mut timer, mut index, mut sprite, airborne_state) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let frames = if airborne_state.0 {
                &air_frames.frames
            } else {
                &ground_frames.frames
            };
            index.0 = (index.0 + 1) % frames.len();
            sprite.image = frames[index.0].clone();
        }
    }
}

fn check_collision(
    mut next: ResMut<NextState<GameState>>,
    audio: Res<Audio>,
    handles: Res<AudioHandles>,
    player_query: Query<&Transform, With<Player>>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
) {
    if let Ok(player) = player_query.single() {
        for obstacle in obstacle_query.iter() {
            if player.translation.distance(obstacle.translation) < 50.0 {
                audio.play(handles.game_over.clone());
                next.set(GameState::GameOver);
            }
        }
    }
}

// ===================================
// ===        MAIN APP             ===
// ===================================
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .init_resource::<AudioHandles>()
        .init_resource::<Score>()
        .init_state::<GameState>()
        .add_plugins(PlayerPlugin)
        .add_plugins(ObstaclePlugin)
        .run();
}
