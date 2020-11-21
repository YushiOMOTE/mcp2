use bevy::{
    asset::AssetServerSettings,
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    prelude::*,
};

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 1000,
            height: 1000,
            ..Default::default()
        })
        .add_resource(AssetServerSettings {
            asset_folder: option_env!("MCP2_PREFIX").unwrap_or("/").to_string(),
        })
        .add_plugins(bevy_webgl2::DefaultPlugins)
        .add_startup_system(setup)
        .init_resource::<TrackInputState>()
        .add_system(track_inputs)
        .add_system(update_ball)
        .run();
}

#[derive(Default)]
struct TrackInputState {
    keys: EventReader<KeyboardInput>,
    cursor: EventReader<CursorMoved>,
    motion: EventReader<MouseMotion>,
    mousebtn: EventReader<MouseButtonInput>,
    scroll: EventReader<MouseWheel>,
}

struct Ball {
    keybinds: KeyBinds,
}

struct KeyBinds {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

#[derive(Default)]
struct BallMotion {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    speed: f32,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ball = asset_server.load("ball.png");

    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteBundle {
            material: materials.add(ball.into()),
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(20.0, 20.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Ball {
            keybinds: KeyBinds {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
            },
        })
        .with(BallMotion {
            speed: 200.0,
            ..Default::default()
        });
}

fn track_inputs(
    mut state: ResMut<TrackInputState>,
    keys: Res<Events<KeyboardInput>>,
    cursor: Res<Events<CursorMoved>>,
    motion: Res<Events<MouseMotion>>,
    mousebtn: Res<Events<MouseButtonInput>>,
    scroll: Res<Events<MouseWheel>>,
    mut query: Query<(&Ball, &mut BallMotion)>,
) {
    for e in state.keys.iter(&keys) {
        for (ball, mut state) in query.iter_mut() {
            match e.key_code {
                Some(k) if k == ball.keybinds.up => {
                    state.up = e.state.is_pressed();
                }
                Some(k) if k == ball.keybinds.down => {
                    state.down = e.state.is_pressed();
                }
                Some(k) if k == ball.keybinds.left => {
                    state.left = e.state.is_pressed();
                }
                Some(k) if k == ball.keybinds.right => {
                    state.right = e.state.is_pressed();
                }
                _ => {}
            }

            if e.state.is_pressed() {
                info!("Key pressed `{:?}`", e.key_code);
            } else {
                info!("Key released `{:?}`", e.key_code);
            }
        }
    }

    for e in state.cursor.iter(&cursor) {
        info!("Cursor at {}", e.position);
    }

    for e in state.motion.iter(&motion) {
        info!("Mouse moved {} pixels", e.delta);
    }

    for e in state.mousebtn.iter(&mousebtn) {
        if e.state.is_pressed() {
            info!("Mouse pressed `{:?}`", e.button);
        } else {
            info!("Mouse released `{:?}`", e.button)
        }
    }

    for e in state.scroll.iter(&scroll) {
        info!("Scrolled direction ({}, {})", e.x, e.y);
    }
}

fn update_ball(time: Res<Time>, mut query: Query<(&BallMotion, &mut Transform)>) {
    for (state, mut transform) in query.iter_mut() {
        if state.up {
            transform.translation.y += time.delta_seconds * state.speed;
        }
        if state.down {
            transform.translation.y -= time.delta_seconds * state.speed;
        }

        if state.right {
            transform.translation.x += time.delta_seconds * state.speed;
        }
        if state.left {
            transform.translation.x -= time.delta_seconds * state.speed;
        }
    }
}
