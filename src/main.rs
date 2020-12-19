use bevy::{
    asset::AssetServerSettings, input::keyboard::KeyboardInput, prelude::*, render::camera::Camera,
};
use derive_new::new;
use serde::{Deserialize, Serialize};

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
        .init_resource::<GameMode>()
        .add_system(track_inputs_system)
        .add_stage_after(stage::UPDATE, "before")
        .add_stage_after(stage::UPDATE, "after")
        .add_system_to_stage("before", move_char_system)
        .add_system_to_stage("before", animate_system)
        .add_system_to_stage("before", gravity_system)
        .add_system_to_stage("after", physics_system)
        .add_system(camera_system)
        .run();
}

#[derive(Default)]
struct TrackInputState {
    keys: EventReader<KeyboardInput>,
}

#[derive(Debug, Default)]
struct GameMode {
    debug_mode: bool,
}

struct Char {
    keybinds: KeyBinds,
}

#[derive(Debug)]
struct KeyBinds {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

#[derive(Debug, Default)]
struct CharMotion {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[derive(Debug)]
struct Gravity;

#[derive(Debug)]
struct Player {
    on_ground: bool,
    size: Vec2,
    velocity: Vec3,
}

#[derive(Debug, new)]
struct Terrain {
    size: Vec2,
    collision: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Tile {
    map: Vec<Vec<u32>>,
}

fn load_tilemap() -> Tile {
    serde_json::from_slice(include_bytes!("tiles.json")).unwrap()
}

const TILE_SIZE: f32 = 32.0;

fn setup_terrain(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    const TEXTURE_WIDTH: f32 = 192.0 * 2.0;
    const TEXTURE_HEIGHT: f32 = 96.0 * 2.0;

    // Load and process tiles texture
    let texture = asset_server.load("textures/tiles.png");
    let mut atlas = TextureAtlas::new_empty(texture, Vec2::new(TEXTURE_WIDTH, TEXTURE_HEIGHT));
    for y in 0..6 {
        for x in 0..12 {
            let x = x as f32 * TILE_SIZE;
            let y = y as f32 * TILE_SIZE;
            atlas.add_texture(bevy::sprite::Rect {
                min: Vec2::new(x, y),
                max: Vec2::new(x + TILE_SIZE, y + TILE_SIZE),
            });
        }
    }

    let atlas_handle = atlases.add(atlas);

    // Load tilemap
    let tile = load_tilemap();
    let xbase = -100.0;
    let ybase = 100.0;

    // Render tiles
    for (x, y, &i) in tile
        .map
        .iter()
        .enumerate()
        .map(|(y, v)| v.iter().enumerate().map(move |(x, i)| (x, y, i)))
        .flatten()
        .filter(|(_, _, &i)| i != 0)
    {
        let x = x as f32 * TILE_SIZE + xbase;
        let y = y as f32 * -TILE_SIZE + ybase;

        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(i - 1),
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                ..Default::default()
            })
            .with(Terrain::new(Vec2::new(TILE_SIZE, TILE_SIZE), true));
    }
}

fn setup_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    // Load character animation
    let texture = asset_server.load("textures/char.png");
    let mut atlas = TextureAtlas::new_empty(texture, Vec2::new(128.0 * 2.0, 240.0 * 2.0));
    for y in 0..15 {
        for x in 0..8 {
            let x = x as f32 * TILE_SIZE;
            let y = y as f32 * TILE_SIZE;
            atlas.add_texture(bevy::sprite::Rect {
                min: Vec2::new(x, y),
                max: Vec2::new(x + TILE_SIZE, y + TILE_SIZE),
            });
        }
    }

    let atlas_handle = atlases.add(atlas);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(2),
            texture_atlas: atlas_handle,
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 0.0)),
            ..Default::default()
        })
        .with(Char {
            keybinds: KeyBinds {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
            },
        })
        .with(CharMotion::default())
        .with(Timer::from_seconds(0.2, true))
        .with(Player {
            on_ground: false,
            size: Vec2::new(TILE_SIZE, TILE_SIZE),
            velocity: Vec3::zero(),
        })
        .with(Gravity);
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    setup_player(commands, &asset_server, &mut atlases);
    setup_terrain(commands, &asset_server, &mut atlases);
}

fn track_inputs_system(
    mut state: ResMut<TrackInputState>,
    mut game_mode: ResMut<GameMode>,
    keys: Res<Events<KeyboardInput>>,
    mut query: Query<(&Char, &mut CharMotion)>,
) {
    for e in state.keys.iter(&keys) {
        for (char, mut state) in query.iter_mut() {
            match e.key_code {
                Some(k) if k == char.keybinds.up => {
                    state.up = e.state.is_pressed();
                }
                Some(k) if k == char.keybinds.down => {
                    state.down = e.state.is_pressed();
                }
                Some(k) if k == char.keybinds.left => {
                    state.left = e.state.is_pressed();
                }
                Some(k) if k == char.keybinds.right => {
                    state.right = e.state.is_pressed();
                }
                Some(k) if k == KeyCode::E => {
                    if !game_mode.debug_mode && e.state.is_pressed() {
                        game_mode.debug_mode = true;
                    }
                }
                Some(k) if k == KeyCode::P => {
                    if game_mode.debug_mode && e.state.is_pressed() {
                        game_mode.debug_mode = false;
                    }
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
}

fn animate_system(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    mut query: Query<(&Player, &mut Timer, &mut TextureAtlasSprite)>,
) {
    for (player, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds);
        if timer.finished {
            if game_mode.debug_mode {
                sprite.index = 1;
            } else if player.velocity.x != 0.0 {
                sprite.index = ((sprite.index as usize + 1) % 6 + 8) as u32;
            } else {
                sprite.index = 40;
            }
        }
    }
}

fn move_char_system(game_mode: Res<GameMode>, mut query: Query<(&mut Player, &CharMotion)>) {
    for (mut player, state) in query.iter_mut() {
        if game_mode.debug_mode {
            if state.up {
                player.velocity.y = 300.0;
            } else if state.down {
                player.velocity.y = -300.0;
            } else {
                player.velocity.y = 0.0;
            }
        } else if state.up && player.on_ground {
            player.velocity.y = 500.0;
            player.on_ground = false;
        }

        if state.right {
            player.velocity.x = 300.0;
        } else if state.left {
            player.velocity.x = -300.0;
        } else {
            player.velocity.x = 0.0;
        }
    }
}

fn gravity_system(game_mode: Res<GameMode>, mut query: Query<&mut Player>) {
    if game_mode.debug_mode {
        return;
    }
    for mut player in query.iter_mut() {
        player.velocity.y -= 9.8;
    }
}

fn camera_system(
    query: Query<(&Player, &Transform)>,
    mut camera: Query<(&mut Camera, &mut Transform)>,
) {
    for (_, player_transform) in query.iter() {
        for (_, mut camera_transform) in camera.iter_mut() {
            camera_transform.translation = player_transform.translation.clone();
        }
    }
}

fn to_rect(translation: &Vec3, size: &Vec2) -> Rect<f32> {
    Rect {
        left: translation.x,
        right: translation.x + size.x,
        bottom: translation.y,
        top: translation.y + size.y,
    }
}

fn physics_system(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform)>,
    mut terrains: Query<(&Terrain, &Transform)>,
) {
    for (mut p, mut pt) in query.iter_mut() {
        let player = to_rect(&pt.translation, &p.size);

        let new_player_pos = pt.translation + time.delta_seconds * p.velocity;
        let new_player = to_rect(&new_player_pos, &p.size);

        let mut possible_y = new_player_pos.y;
        let mut possible_x = new_player_pos.x;
        let mut new_velocity = p.velocity.clone();

        for (t, tt) in terrains.iter_mut() {
            if !t.collision {
                continue;
            }

            let terrain = to_rect(&tt.translation, &t.size);

            if new_player.right <= terrain.left
                || terrain.right <= new_player.left
                || new_player.top <= terrain.bottom
                || terrain.top <= new_player.bottom
            {
                // no collision
                continue;
            }

            // can collide; constraint player position

            // time until top/bottom collision
            let ty = if p.velocity.y < 0.0 && terrain.top <= player.bottom {
                (terrain.top - player.bottom) / p.velocity.y
            } else if p.velocity.y > 0.0 && player.top <= terrain.bottom {
                (terrain.bottom - player.top) / p.velocity.y
            } else {
                f32::INFINITY
            };

            // time until left/right collision
            let tx = if p.velocity.x < 0.0 && terrain.right <= player.left {
                (terrain.right - player.left) / p.velocity.x
            } else if p.velocity.x > 0.0 && player.right <= terrain.left {
                (terrain.left - player.right) / p.velocity.x
            } else {
                f32::INFINITY
            };

            p.on_ground = ty == 0.0;

            if ty < tx {
                // top/bottom collides before left/right collides

                if p.velocity.y < 0.0 {
                    // player bottom collides
                    possible_y = possible_y.max(terrain.top);
                } else {
                    // player top collides
                    possible_y = possible_y.min(terrain.bottom - p.size.y);
                }

                new_velocity.y = 0.0;
            } else {
                // left/right collides before top/bottom collides

                if p.velocity.x < 0.0 {
                    // player left collides
                    possible_x = possible_x.max(terrain.right);
                } else {
                    // player right collides
                    possible_x = possible_x.min(terrain.left - p.size.x);
                }

                new_velocity.x = 0.0;
            }
        }

        pt.translation.x = possible_x;
        pt.translation.y = possible_y;
        p.velocity = new_velocity;
    }
}
