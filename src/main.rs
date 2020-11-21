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
        .add_resource(AssetServerSettings {
            asset_folder: option_env!("MCP2_PREFIX").unwrap_or("/").to_string(),
        })
        .add_plugins(bevy_webgl2::DefaultPlugins)
        .add_startup_system(setup)
        .init_resource::<TrackInputState>()
        .add_system(track_inputs)
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
) {
    for e in state.keys.iter(&keys) {
        if e.state.is_pressed() {
            info!("Key pressed `{:?}`", e.key_code);
        } else {
            info!("Key released `{:?}`", e.key_code);
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
