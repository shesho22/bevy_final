use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource, AudioControl};

#[derive(Component)]
struct Player {
    velocity: Vec2,
}

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Obstacle;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

// Recurso para guardar los handles de audio
#[derive(Resource, Default)]
struct AudioHandles {
    jump: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut audio_handles: ResMut<AudioHandles>,
) {
    commands.spawn(Camera2d);

    // Cargar sonidos
    audio_handles.jump = asset_server.load("sounds/jump.ogg");
    audio_handles.game_over = asset_server.load("sounds/game-over.ogg");

    // Jugador
    commands.spawn((
        Text2d::new("D"),
        TextFont {
            font_size: 60.0,
            font: default(),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(-200.0, -100.0, 0.0)),
        Player { velocity: Vec2::ZERO },
    ));

    // Piso
    commands.spawn((
        Text2d::new(
            "==========================================================================================================================================================",
        ),
        TextFont {
            font_size: 20.0,
            font: default(),
            ..default()
        },
        TextColor(Color::BLACK),
        Transform::from_translation(Vec3::new(0.0, -120.0, 0.0)),
        Floor,
    ));

    // Obstáculos iniciales
    for i in 0..5 {
        commands.spawn((
            Text2d::new("X"),
            TextFont {
                font_size: 40.0,
                font: default(),
                ..default()
            },
            TextColor(Color::BLACK),
            Transform::from_translation(Vec3::new(400.0 + i as f32 * 200.0, -90.0, 0.0)),
            Obstacle,
        ));
    }
}

// Sistema para iniciar el juego desde el menú
fn start_game(input: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if input.any_pressed([KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight]) {
        next_state.set(GameState::Playing);
    }
}

// Movimiento del jugador (solo durante Playing) con sonido
fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
    mut player_query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    if let Ok((mut transform, mut player)) = player_query.single_mut() {
        let normal_gravity = -500.0;
        let fast_fall_gravity = -1200.0;
        let jump_speed = 300.0;

        let floor_y = -120.0;
        let player_half_height = 30.0;
        let on_floor = transform.translation.y <= floor_y + player_half_height + 0.1;

        // Saltar solo si está en el piso
        if input.pressed(KeyCode::ArrowUp) && on_floor {
            player.velocity.y = jump_speed;

            // Reproducir sonido de salto
            audio.play(audio_handles.jump.clone());
        }

        // Gravedad normal o rápida si presionamos abajo
        let gravity = if input.pressed(KeyCode::ArrowDown) {
            fast_fall_gravity
        } else {
            normal_gravity
        };

        let delta = time.delta().as_secs_f32();
        player.velocity.y += gravity * delta;

        // Mover jugador
        transform.translation.x += player.velocity.x * delta;
        transform.translation.y += player.velocity.y * delta;

        // Evitar que pase por debajo del piso
        if transform.translation.y < floor_y + player_half_height {
            transform.translation.y = floor_y + player_half_height;
            player.velocity.y = 0.0;
        }
    }
}

// Movimiento de los obstáculos hacia el jugador
fn move_obstacles(time: Res<Time>, mut query: Query<&mut Transform, With<Obstacle>>) {
    let speed = 200.0;
    let delta = time.delta().as_secs_f32();
    for mut transform in query.iter_mut() {
        transform.translation.x -= speed * delta;
        if transform.translation.x < -500.0 {
            transform.translation.x = 500.0;
        }
    }
}

// Detección de colisión jugador‑obstáculo → pasa a GameOver con sonido
fn check_collision(
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Transform, With<Player>>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
    audio: Res<Audio>,
    audio_handles: Res<AudioHandles>,
) {
    if let Ok(player_transform) = player_query.single() {
        for obstacle_transform in obstacle_query.iter() {
            let distance = player_transform.translation.distance(obstacle_transform.translation);
            if distance < 40.0 {
                // Reproducir sonido de Game Over
                audio.play(audio_handles.game_over.clone());
                next_state.set(GameState::GameOver);
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .init_resource::<AudioHandles>()
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Menu), setup)
        .add_systems(Update, start_game.run_if(in_state(GameState::Menu)))
        .add_systems(Update, move_player.run_if(in_state(GameState::Playing)))
        .add_systems(Update, move_obstacles.run_if(in_state(GameState::Playing)))
        .add_systems(Update, check_collision.run_if(in_state(GameState::Playing)))
        .run();
}
