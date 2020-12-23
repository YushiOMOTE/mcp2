use bevy::{
    asset::AssetServerSettings, input::keyboard::KeyboardInput, prelude::*, render::camera::Camera,
    render::camera::OrthographicProjection,
};
use derive_new::new;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod atlas;

use crate::atlas::AtlasBuilder;

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
        .add_startup_system(setup_enemies)
        .add_startup_system(setup_player)
        .add_startup_system(setup_terrain)
        .init_resource::<TrackInputState>()
        .init_resource::<GameMode>()
        .init_resource::<TileInfo>()
        .add_system(track_inputs_system)
        .add_stage_after(stage::UPDATE, "before")
        .add_stage_after(stage::UPDATE, "after")
        .add_system_to_stage("before", load_terrain_system)
        .add_system_to_stage("before", move_char_system)
        .add_system_to_stage("before", random_walk_system)
        .add_system_to_stage("before", random_attack_system)
        .add_system_to_stage("before", animate_system)
        .add_system_to_stage("before", gravity_system)
        .add_system_to_stage("before", attack_move_system)
        .add_system_to_stage("after", physics_system)
        .add_system_to_stage("after", attack_collision_system)
        .add_system(camera_system)
        .add_system(shoot_system)
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
    life: u32,
    attack_atlas_handle: Handle<TextureAtlas>,
    attack_timer: Timer,
}

#[derive(Debug)]
struct KeyBinds {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    attack: KeyCode,
}

#[derive(Debug, Default)]
struct CharMotion {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    attack: bool,
}

#[derive(Debug)]
struct Gravity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    init_dir: Dir,
    state: State,
    velocity: Vec3,
    size: Vec2,
    on_ground: bool,
}

impl Char {
    fn flip(&mut self, transform: &mut Transform) {
        if self.dir != self.init_dir {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        } else {
            transform.rotation = Quat::default();
        }
    }
}

#[derive(Debug)]
struct Enemy {
    life: u32,
}

#[derive(Debug)]
struct Attack {
    velocity: Vec2,
}

#[derive(Debug)]
struct EnemyAttack;

#[derive(Debug)]
struct PlayerAttack;

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

fn shoot_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(&mut Player, &Char, &CharMotion, &Transform)>,
) {
    for (mut player, ch, state, transform) in query.iter_mut() {
        player.attack_timer.tick(time.delta_seconds);
        if player.attack_timer.finished && state.attack {
            player.attack_timer.reset();

            let x = if ch.dir == Dir::Right { 500.0 } else { -500.0 };

            commands
                .spawn(SpriteSheetBundle {
                    sprite: TextureAtlasSprite::new(0),
                    texture_atlas: player.attack_atlas_handle.clone(),
                    transform: transform.clone(),
                    ..Default::default()
                })
                .with(Attack {
                    velocity: Vec2::new(x, 0.0),
                })
                .with(PlayerAttack);
        }
    }
}

#[derive(Debug)]
struct RandomAttack {
    attack_index: u32,
    timer: Timer,
    atlas_handle: Handle<TextureAtlas>,
}

fn random_attack_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(&mut RandomAttack, &Transform)>,
) {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for (mut attack, transform) in query.iter_mut() {
        attack.timer.tick(time.delta_seconds);
        if attack.timer.finished {
            if rng.gen_range(0.0..1.0) > 0.6 {
                for i in 0..8 {
                    let pi = 2.0 * std::f32::consts::PI / 8.0 * i as f32;
                    let speed = 100.0;

                    commands
                        .spawn(SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(attack.attack_index),
                            texture_atlas: attack.atlas_handle.clone(),
                            transform: transform.clone(),
                            ..Default::default()
                        })
                        .with(Attack {
                            velocity: Vec2::new(speed * pi.sin(), speed * pi.cos()),
                        })
                        .with(EnemyAttack);
                }
            }
        }
    }
}

fn attack_move_system(time: Res<Time>, mut query: Query<(&Attack, &mut Transform)>) {
    for (attack, mut transform) in query.iter_mut() {
        transform.translation.x += time.delta_seconds * attack.velocity.x;
        transform.translation.y += time.delta_seconds * attack.velocity.y;
    }
}

fn attack_collision_system(
    commands: &mut Commands,
    mut players: Query<(&mut Player, &Char, &Transform)>,
    mut enemies: Query<(Entity, &mut Enemy, &Char, &Transform)>,
    enemy_attacks: Query<(Entity, &EnemyAttack, &Transform)>,
    player_attacks: Query<(Entity, &PlayerAttack, &Transform)>,
) {
    for (mut player, ch, transform) in players.iter_mut() {
        for (e, _, attack_transform) in enemy_attacks.iter() {
            let min_x = transform.translation.x;
            let min_y = transform.translation.y;
            let max_x = transform.translation.x + ch.size.x;
            let max_y = transform.translation.y + ch.size.y;

            let amin_x = attack_transform.translation.x;
            let amin_y = attack_transform.translation.y;
            let amax_x = amin_x + 16.0;
            let amax_y = amin_y + 16.0;

            if max_x < amin_x || amax_x < min_x || max_y < amin_y || amax_y < min_y {
                continue;
            }

            player.life -= 1;
            if player.life == 0 {
                panic!("gameover!");
            }

            commands.despawn(e);
        }
    }

    for (ee, mut enemy, ch, transform) in enemies.iter_mut() {
        for (e, _, attack_transform) in player_attacks.iter() {
            let min_x = transform.translation.x;
            let min_y = transform.translation.y;
            let max_x = transform.translation.x + ch.size.x;
            let max_y = transform.translation.y + ch.size.y;

            let amin_x = attack_transform.translation.x;
            let amin_y = attack_transform.translation.y;
            let amax_x = amin_x + 16.0;
            let amax_y = amin_y + 16.0;

            if max_x < amin_x || amax_x < min_x || max_y < amin_y || amax_y < min_y {
                continue;
            }

            enemy.life -= 1;
            if enemy.life == 0 {
                commands.despawn(ee);
            }

            commands.despawn(e);
        }
    }
}

#[derive(Debug)]
struct RandomWalk {
    timer: Timer,
    move_possibility: f32,
    jump_possibility: f32,
}

fn random_walk_system(
    time: Res<Time>,
    mut query: Query<(&mut Char, &mut RandomWalk, &mut Transform)>,
) {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for (mut ch, mut walk, _) in query.iter_mut() {
        walk.timer.tick(time.delta_seconds);
        if walk.timer.finished {
            if walk.jump_possibility > rng.gen_range(0.0..1.0) {
                ch.velocity.y = 200.0;
            }
            if walk.move_possibility > rng.gen_range(0.0..1.0) {
                ch.velocity.x = rng.gen_range(-100.0..100.0);
            } else {
                ch.velocity.x = 0.0;
            }
        }
    }
}

#[derive(Debug, new)]
struct Terrain {
    size: Vec2,
    collision: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct TileMap {
    map: Vec<Vec<u32>>,
}

#[derive(Debug, Default)]
struct TileInfo {
    center: Vec3,
    loaded: Vec<(usize, usize, u32, Option<Entity>)>,
    atlas_handle: Handle<TextureAtlas>,
    timer: Timer,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct EnemyInfo {
    user: String,
    lgtm: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct EnemyList {
    enemies: Vec<EnemyInfo>,
}

fn load_enemy_list() -> EnemyList {
    serde_json::from_slice(include_bytes!("enemies.json")).unwrap()
}

fn load_tilemap() -> TileMap {
    serde_json::from_slice(include_bytes!("tiles.json")).unwrap()
}

fn setup_terrain(
    asset_server: Res<AssetServer>,
    mut tileinfo: ResMut<TileInfo>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let atlas_handle = AtlasBuilder::load(
        &asset_server,
        Vec2::new(16.0, 16.0),
        Vec2::new(176.0, 135.0),
        "textures/tiles.png",
    )
    .build(&mut atlases);

    tileinfo.loaded = load_tilemap()
        .map
        .into_iter()
        .enumerate()
        .map(|(y, v)| v.into_iter().enumerate().map(move |(x, i)| (x, y, i, None)))
        .flatten()
        .filter(|(_, _, i, _)| *i != 0)
        .collect();
    tileinfo.atlas_handle = atlas_handle;
    tileinfo.timer = Timer::from_seconds(0.2, true);
}

fn setup_player(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let atlas_handle = AtlasBuilder::load(
        &asset_server,
        Vec2::new(32.0, 32.0),
        Vec2::new(160.0, 32.0),
        "textures/char.png",
    )
    .padding(Vec2::new(0.0, 0.0))
    .scale(Vec2::splat(1.0 / 2.0))
    .build(&mut atlases);

    let attack_atlas_handle = AtlasBuilder::load(
        &asset_server,
        Vec2::new(32.0, 32.0),
        Vec2::new(32.0, 32.0),
        "textures/attack.png",
    )
    .padding(Vec2::new(0.0, 0.0))
    .scale(Vec2::splat(0.5))
    .build(&mut atlases);

    let mut animate_map = HashMap::new();
    animate_map.insert(State::Stop, vec![0]);
    animate_map.insert(State::Run, (1..5).collect());
    animate_map.insert(State::Jump, vec![0]);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(8),
            texture_atlas: atlas_handle,
            transform: Transform::from_translation(Vec3::new(1000.0, -500.0, 0.0)),
            ..Default::default()
        })
        .with(Player {
            keybinds: KeyBinds {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
                attack: KeyCode::Space,
            },
            life: 10,
            attack_atlas_handle,
            attack_timer: Timer::from_seconds(0.2, false),
        })
        .with(CharMotion::default())
        .with(Timer::from_seconds(0.2, true))
        .with(Char {
            dir: Dir::Right,
            init_dir: Dir::Right,
            state: State::Stop,
            velocity: Vec3::zero(),
            size: Vec2::new(16.0, 16.0),
            on_ground: false,
        })
        .with(Gravity)
        .with(Animate::new(animate_map));
}

fn setup_enemies(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let atlas_handle = AtlasBuilder::load(
        &asset_server,
        Vec2::new(32.0, 32.0),
        Vec2::new(160.0, 32.0),
        "textures/enemy.png",
    )
    .padding(Vec2::new(0.0, 0.0))
    .scale(Vec2::splat(1.0))
    .build(&mut atlases);

    let enemies = load_enemy_list();

    let attack_atlas_handle = AtlasBuilder::load(
        &asset_server,
        Vec2::new(32.0, 32.0),
        Vec2::new(32.0 * enemies.enemies.len() as f32, 32.0),
        "textures/enemies_sheet.png",
    )
    .padding(Vec2::new(0.0, 0.0))
    .scale(Vec2::splat(0.4))
    .build(&mut atlases);

    let mut animate_map = HashMap::new();
    animate_map.insert(State::Stop, vec![0]);
    animate_map.insert(State::Run, (1..5).collect());
    animate_map.insert(State::Jump, vec![1, 3]);

    for (i, e) in enemies.enemies.into_iter().take(3).enumerate() {
        let base_transform =
            Transform::from_translation(Vec3::new(180.0 * i as f32 + 500.0, -500.0, 0.0));

        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(15),
                texture_atlas: atlas_handle.clone(),
                transform: base_transform,
                ..Default::default()
            })
            .with(Timer::from_seconds(0.2, true))
            .with(Enemy { life: e.lgtm })
            .with(Char {
                dir: Dir::Right,
                init_dir: Dir::Left,
                state: State::Stop,
                velocity: Vec3::zero(),
                size: Vec2::new(32.0, 32.0),
                on_ground: false,
            })
            .with(Animate::new(animate_map.clone()))
            .with(RandomWalk {
                timer: Timer::from_seconds(1.0, true),
                move_possibility: 0.3,
                jump_possibility: 0.3,
            })
            .with(RandomAttack {
                timer: Timer::from_seconds(1.0, true),
                attack_index: i as u32,
                atlas_handle: attack_atlas_handle.clone(),
            })
            .with(Gravity);
    }
}

fn load_terrain_system(
    time: Res<Time>,
    commands: &mut Commands,
    mut tileinfo: ResMut<TileInfo>,
    camera: Query<(&Camera, &OrthographicProjection, &Transform)>,
    attacks: Query<(Entity, &Attack, &Transform)>,
) {
    let (_, proj, center) = camera.iter().next().unwrap();

    tileinfo.timer.tick(time.delta_seconds);
    if !tileinfo.timer.finished {
        return;
    }

    if center.translation == tileinfo.center {
        return;
    }
    tileinfo.center = center.translation;

    let min_x = center.translation.x + proj.left * center.scale.x * 1.5;
    let min_y = center.translation.y + proj.bottom * center.scale.y * 1.5;
    let max_x = center.translation.x + proj.right * center.scale.x * 1.5;
    let max_y = center.translation.y + proj.top * center.scale.y * 1.5;

    let handle = tileinfo.atlas_handle.clone();

    let mut loaded_count = 0;
    let mut unloaded_count = 0;
    let mut total = 0;

    for (x, y, i, loaded) in tileinfo.loaded.iter_mut() {
        let x = *x as f32 * 16.0;
        let y = *y as f32 * -16.0;

        if x >= min_x && x < max_x && y >= min_y && y < max_y {
            if loaded.is_none() {
                loaded_count += 1;

                *loaded = Some(
                    commands
                        .spawn(SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(*i - 1),
                            texture_atlas: handle.clone(),
                            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                            ..Default::default()
                        })
                        .with(Terrain::new(Vec2::new(16.0, 16.0), true))
                        .current_entity()
                        .unwrap(),
                );
            }
        } else {
            if let Some(entity) = loaded.take() {
                unloaded_count += 1;
                commands.despawn(entity);
            }
        }

        if loaded.is_some() {
            total += 1;
        }
    }

    info!(
        "Loaded: {}, Unloaded: {} (Current: {})",
        loaded_count, unloaded_count, total
    );

    for (e, _, transform) in attacks.iter() {
        let x = transform.translation.x;
        let y = transform.translation.y;
        if x >= min_x && x < max_x && y >= min_y && y < max_y {
            continue;
        }
        commands.despawn(e);
    }
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
                Some(k) if k == player.keybinds.attack => {
                    state.attack = e.state.is_pressed();
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
            ch.velocity.y = 300.0;
            ch.on_ground = false;
        }

        if state.right {
            ch.velocity.x = 100.0;
        } else if state.left {
            ch.velocity.x = -100.0;
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
            camera_transform.scale = Vec3::splat(0.3);
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
            } else {
                ch.dir = Dir::Left;
            }
            ch.flip(&mut cht);
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
