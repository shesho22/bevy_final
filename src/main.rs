use bevy::{app::Animation, prelude::*};
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
struct Player {
    velocity: Vec2,
}

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Obstacle {
    airborne: bool,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct FrameIndex(usize);

// ===================================
// ===            ESTADOS           ===
// ===================================
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

// ===================================
// ===           RECURSOS           ===
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
// ===         SETUP INICIAL        ===
// ===================================
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut audio_handles: ResMut<AudioHandles>,
) {
    // Cámara
    commands.spawn((Camera2d, Transform::from_scale(Vec3::splat(0.8))));

    // Fondo
    let background = asset_server.load("backgrounds/background.png");
    commands.spawn((
        Sprite::from_image(background),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
    ));

    // Cargar sonidos
    audio_handles.jump = asset_server.load("sounds/jump.ogg");
    audio_handles.game_over = asset_server.load("sounds/game-over.ogg");

    // === FRAMES DEL JUGADOR ===
    let frames = vec![
        asset_server.load("sprites/character1.png"),
        asset_server.load("sprites/character2.png"),
        asset_server.load("sprites/character3.png"),
        asset_server.load("sprites/character4.png"),
    ];
    commands.insert_resource(PlayerFrames {
        frames: frames.clone(),
    });

    // === FRAMES DE OBSTÁCULOS ===
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

    commands.insert_resource(ObstacleGroundFrames {
        frames: ground_frames.clone(),
    });
    commands.insert_resource(ObstacleAirFrames {
        frames: air_frames.clone(),
    });

    // Texturas base
    let ground_obstacle = ground_frames[0].clone();
    let air_obstacle = air_frames[0].clone();
    commands.insert_resource(ObstacleTextures {
        ground: ground_obstacle.clone(),
        air: air_obstacle.clone(),
    });

    // === JUGADOR ===
    commands.spawn((
        Sprite::from_image(frames[0].clone()),
        Transform::from_translation(PLAYER_START_POS).with_scale(Vec3::splat(0.5)),
        Player { velocity: Vec2::ZERO },
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        FrameIndex(0),
    ));

    // === PISO ===
    let floor_texture = asset_server.load("sprites/floor.png");
    let num_tiles = 20;
    let tile_width = 64.0;
    let tile_scale = 1.0;
    let y_offset = -40.0;

    for i in 0..num_tiles {
        let x = (i as f32 * tile_width * tile_scale)
            - (num_tiles as f32 * tile_width * tile_scale / 2.0);

        commands.spawn((
            Sprite::from_image(floor_texture.clone()),
            Transform {
                translation: Vec3::new(x, FLOOR_Y + y_offset, 0.0),
                scale: Vec3::splat(tile_scale),
                ..default()
            },
            Floor,
        ));
    }

    // === OBSTÁCULOS INICIALES ===
    let mut rng = rand::thread_rng();
    for i in 0..5 {
        let x = OBSTACLE_START_X + i as f32 * OBSTACLE_SPACING;
        let airborne = rng.gen_bool(0.5);
        let y = if airborne { FLOOR_Y + 90.0 } else { FLOOR_Y + 10.0 };
        let texture = if airborne {
            air_frames[0].clone()
        } else {
            ground_frames[0].clone()
        };

        commands.spawn((
            Sprite::from_image(texture),
            Transform::from_translation(Vec3::new(x, y, 0.0)).with_scale(Vec3::splat(0.7)),
            Obstacle { airborne },
            AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
            FrameIndex(0),
        ));
    }

    // === TEXTO DE PUNTAJE ===
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            font: default(),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(-350.0, 200.0, 1.0)),
        ScoreText,
    ));
}

// ===================================
// ===         SISTEMAS DE JUEGO    ===
// ===================================
fn start_game(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.any_pressed([KeyCode::ArrowUp, KeyCode::Space, KeyCode::Enter]) {
        next_state.set(GameState::Playing);
    }
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    mut query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    if let Ok((mut transform, mut player)) = query.single_mut() {
        let delta = time.delta().as_secs_f32();
        let gravity = if input.pressed(KeyCode::ArrowDown) { -2000.0 } else { -500.0 };
        let jump_speed = 350.0;

        let on_floor = transform.translation.y <= FLOOR_Y + PLAYER_HEIGHT / 2.0 + 0.1;

        if input.just_pressed(KeyCode::ArrowUp) && on_floor {
            player.velocity.y = jump_speed;
            audio.play(audio_handles.jump.clone());
        }

        player.velocity.y += gravity * delta;
        transform.translation.y += player.velocity.y * delta;

        if transform.translation.y < FLOOR_Y + PLAYER_HEIGHT / 2.0 {
            transform.translation.y = FLOOR_Y + PLAYER_HEIGHT / 2.0;
            player.velocity.y = 0.0;
        }
    }
}

fn move_obstacles(
    time: Res<Time>,
    obstacle_textures: Res<ObstacleTextures>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut Obstacle)>,
) {
    let delta = time.delta().as_secs_f32();
    let mut rng = rand::thread_rng();

    for (mut transform, mut sprite, mut obstacle) in query.iter_mut() {
        transform.translation.x -= OBSTACLE_SPEED * delta;

        // Reposición
        if transform.translation.x < OBSTACLE_MIN_X {
            obstacle.airborne = rng.gen_bool(0.5);
            transform.translation.x = OBSTACLE_RESET_X;
            transform.translation.y = if obstacle.airborne {
                FLOOR_Y + 90.0
            } else {
                FLOOR_Y + 10.0
            };
            sprite.image = if obstacle.airborne {
                obstacle_textures.air.clone()
            } else {
                obstacle_textures.ground.clone()
            };
        }
    }
}

fn check_collision(
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, With<Player>>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
) {
    if let Ok(player_tf) = player_query.single() {
        for obstacle_tf in obstacle_query.iter() {
            let distance = player_tf.translation.distance(obstacle_tf.translation);
            if distance < 50.0 {
                audio.play(audio_handles.game_over.clone());
                next_state.set(GameState::GameOver);
            }
        }
    }
}

// ===================================
// ===         SISTEMA PUNTAJE      ===
// ===================================
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
// ===         SISTEMAS FRAMES      ===
// ===================================
fn animate_player_frames(
    time: Res<Time>,
    player_frames: Res<PlayerFrames>,
    mut query: Query<(&mut AnimationTimer, &mut FrameIndex, &mut Sprite), With<Player>>,
) {
    for (mut timer, mut frame_index, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            frame_index.0 = (frame_index.0 + 1) % player_frames.frames.len();
            sprite.image = player_frames.frames[frame_index.0].clone();
        }
    }
}

fn animate_obstacle_frames(
    time: Res<Time>,
    ground_frames: Res<ObstacleGroundFrames>,
    air_frames: Res<ObstacleAirFrames>,
    mut query: Query<(&mut AnimationTimer, &mut FrameIndex, &mut Sprite, &Obstacle)>,
) {
    for (mut timer, mut frame_index, mut sprite, obstacle) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let frames = if obstacle.airborne {
                &air_frames.frames
            } else {
                &ground_frames.frames
            };
            frame_index.0 = (frame_index.0 + 1) % frames.len();
            sprite.image = frames[frame_index.0].clone();
        }
    }
}

// ===================================
// ===         FUNCIÓN MAIN         ===
// ===================================
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .init_resource::<AudioHandles>()
        .init_resource::<Score>()
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Menu), setup)
        .add_systems(Update, start_game.run_if(in_state(GameState::Menu)))
        .add_systems(Update, move_player.run_if(in_state(GameState::Playing)))
        .add_systems(Update, move_obstacles.run_if(in_state(GameState::Playing)))
        .add_systems(Update, check_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_score.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_player_frames.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_obstacle_frames.run_if(in_state(GameState::Playing)))
        .run();
}
