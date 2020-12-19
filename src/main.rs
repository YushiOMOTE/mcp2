use bevy::{
    asset::AssetServerSettings, input::keyboard::KeyboardInput, prelude::*, render::camera::Camera,
};
use derive_new::new;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

struct Player {
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

#[derive(Debug, Clone, Copy)]
enum Dir {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    Jump,
    Run,
    Stop,
}

#[derive(Debug)]
struct Char {
    dir: Dir,
    state: State,
    velocity: Vec3,
    size: Vec2,
    on_ground: bool,
}

#[derive(Debug, new)]
struct Animate {
    animation: HashMap<State, Vec<u32>>,
    #[new(default)]
    index: usize,
}

impl Animate {
    fn next(&mut self, state: State) -> u32 {
        let animation = self.animation.get(&state).unwrap();
        let index = animation[self.index % animation.len()];
        self.index = (self.index + 1) % animation.len();
        index
    }
}

#[derive(Debug)]
struct Enemy {
    size: Vec2,
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

    let mut animate_map = HashMap::new();

    animate_map.insert(State::Stop, vec![40]);
    animate_map.insert(State::Run, (0..6).map(|v| v + 8).collect());
    animate_map.insert(State::Jump, vec![40]);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(2),
            texture_atlas: atlas_handle,
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 0.0)),
            ..Default::default()
        })
        .with(Player {
            keybinds: KeyBinds {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
            },
        })
        .with(CharMotion::default())
        .with(Timer::from_seconds(0.2, true))
        .with(Char {
            dir: Dir::Right,
            state: State::Stop,
            velocity: Vec3::zero(),
            size: Vec2::new(TILE_SIZE, TILE_SIZE),
            on_ground: false,
        })
        .with(Gravity)
        .with(Animate::new(animate_map));
}

fn setup_enemies(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    // Load enemy animation
    let texture = asset_server.load("textures/slime.png");
    let mut atlas = TextureAtlas::new_empty(texture, Vec2::new(240.0 * 2.0, 80.0 * 2.0));
    for y in 0..5 {
        for x in 0..15 {
            let x = x as f32 * TILE_SIZE;
            let y = if y == 0 { 0.0 } else { y as f32 * TILE_SIZE };
            atlas.add_texture(bevy::sprite::Rect {
                min: Vec2::new(x, y),
                max: Vec2::new(x + TILE_SIZE, y + TILE_SIZE),
            });
        }
    }

    let atlas_handle = atlases.add(atlas);

    let mut animate_map = HashMap::new();

    animate_map.insert(State::Stop, (0..15).map(|v| v + 15).collect());
    animate_map.insert(State::Run, (0..15).map(|v| v + 15).collect());
    animate_map.insert(State::Jump, vec![16]);

    for i in 0..10 {
        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(15),
                texture_atlas: atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(180.0 * i as f32, 500.0, 0.0)),
                ..Default::default()
            })
            .with(Timer::from_seconds(0.2, true))
            .with(Char {
                dir: Dir::Right,
                state: State::Stop,
                velocity: Vec3::zero(),
                size: Vec2::new(TILE_SIZE, TILE_SIZE),
                on_ground: false,
            })
            .with(Animate::new(animate_map.clone()))
            .with(Gravity);
    }
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    setup_player(commands, &asset_server, &mut atlases);
    setup_enemies(commands, &asset_server, &mut atlases);
    setup_terrain(commands, &asset_server, &mut atlases);
}

fn track_inputs_system(
    mut state: ResMut<TrackInputState>,
    keys: Res<Events<KeyboardInput>>,
    mut query: Query<(&Player, &mut CharMotion)>,
) {
    for e in state.keys.iter(&keys) {
        for (player, mut state) in query.iter_mut() {
            match e.key_code {
                Some(k) if k == player.keybinds.up => {
                    state.up = e.state.is_pressed();
                }
                Some(k) if k == player.keybinds.down => {
                    state.down = e.state.is_pressed();
                }
                Some(k) if k == player.keybinds.left => {
                    state.left = e.state.is_pressed();
                }
                Some(k) if k == player.keybinds.right => {
                    state.right = e.state.is_pressed();
                }
                // TODO: Temporarily disbale because this is confusing
                // Some(k) if k == KeyCode::E => {
                //     if !game_mode.debug_mode && e.state.is_pressed() {
                //         game_mode.debug_mode = true;
                //     }
                // }
                // Some(k) if k == KeyCode::P => {
                //     if game_mode.debug_mode && e.state.is_pressed() {
                //         game_mode.debug_mode = false;
                //     }
                // }
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
    mut query: Query<(&Char, &mut Animate, &mut Timer, &mut TextureAtlasSprite)>,
) {
    for (ch, mut animate, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds);
        if timer.finished {
            sprite.index = animate.next(ch.state);
        }
    }
}

fn move_char_system(game_mode: Res<GameMode>, mut query: Query<(&mut Char, &Player, &CharMotion)>) {
    for (mut ch, _, state) in query.iter_mut() {
        if game_mode.debug_mode {
            if state.up {
                ch.velocity.y = 300.0;
            } else if state.down {
                ch.velocity.y = -300.0;
            } else {
                ch.velocity.y = 0.0;
            }
        } else if state.up && ch.on_ground {
            ch.velocity.y = 500.0;
            ch.on_ground = false;
        }

        if state.right {
            ch.velocity.x = 300.0;
        } else if state.left {
            ch.velocity.x = -300.0;
        } else {
            ch.velocity.x = 0.0;
        }
    }
}

fn gravity_system(game_mode: Res<GameMode>, mut query: Query<&mut Char>) {
    if game_mode.debug_mode {
        return;
    }
    for mut ch in query.iter_mut() {
        ch.velocity.y -= 9.8;
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
    mut query: Query<(&mut Char, &mut Transform)>,
    mut terrains: Query<(&Terrain, &Transform)>,
) {
    for (mut ch, mut cht) in query.iter_mut() {
        let old_ch = to_rect(&cht.translation, &ch.size);

        let new_ch_pos = cht.translation + time.delta_seconds * ch.velocity;
        let new_ch = to_rect(&new_ch_pos, &ch.size);

        let mut possible_y = new_ch_pos.y;
        let mut possible_x = new_ch_pos.x;
        let mut new_velocity = ch.velocity.clone();

        ch.on_ground = false;

        for (t, tt) in terrains.iter_mut() {
            if !t.collision {
                continue;
            }

            let terrain = to_rect(&tt.translation, &t.size);

            if new_ch.right <= terrain.left
                || terrain.right <= new_ch.left
                || new_ch.top <= terrain.bottom
                || terrain.top <= new_ch.bottom
            {
                // no collision
                continue;
            }

            // can collide; constraint character position

            // time until top/bottom collision
            let ty = if ch.velocity.y < 0.0 && terrain.top <= old_ch.bottom {
                (terrain.top - old_ch.bottom) / ch.velocity.y
            } else if ch.velocity.y > 0.0 && old_ch.top <= terrain.bottom {
                (terrain.bottom - old_ch.top) / ch.velocity.y
            } else {
                f32::INFINITY
            };

            // time until left/right collision
            let tx = if ch.velocity.x < 0.0 && terrain.right <= old_ch.left {
                (terrain.right - old_ch.left) / ch.velocity.x
            } else if ch.velocity.x > 0.0 && old_ch.right <= terrain.left {
                (terrain.left - old_ch.right) / ch.velocity.x
            } else {
                f32::INFINITY
            };

            ch.on_ground = ty == 0.0;

            if ty <= tx {
                // top/bottom collides before left/right collides

                if ch.velocity.y < 0.0 {
                    // character bottom collides
                    possible_y = possible_y.max(terrain.top);
                } else {
                    // character top collides
                    possible_y = possible_y.min(terrain.bottom - ch.size.y);
                }

                new_velocity.y = 0.0;
            } else {
                // left/right collides before top/bottom collides

                if ch.velocity.x < 0.0 {
                    // character left collides
                    possible_x = possible_x.max(terrain.right);
                } else {
                    // character right collides
                    possible_x = possible_x.min(terrain.left - ch.size.x);
                }

                new_velocity.x = 0.0;
            }
        }

        cht.translation.x = possible_x;
        cht.translation.y = possible_y;
        ch.velocity = new_velocity;

        if ch.velocity.x != 0.0 {
            if ch.velocity.x > 0.0 {
                ch.dir = Dir::Right;
                cht.rotation = Quat::default();
            } else {
                ch.dir = Dir::Left;
                cht.rotation = Quat::from_rotation_y(std::f32::consts::PI);
            }
        }

        if ch.velocity.y != 0.0 {
            ch.state = State::Jump;
        } else if ch.velocity.x != 0.0 {
            ch.state = State::Run;
        } else {
            ch.state = State::Stop;
        }
    }
}
