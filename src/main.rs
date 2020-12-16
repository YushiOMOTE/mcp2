use bevy::{
    asset::AssetServerSettings,
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
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
        .add_system(track_inputs_system)
        .add_stage_after(stage::UPDATE, "before")
        .add_stage_after(stage::UPDATE, "after")
        .add_system_to_stage("before", move_char_system)
        .add_system_to_stage("before", animate_system)
        .add_system_to_stage("before", gravity_system)
        .add_system_to_stage("after", physics_system)
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

struct Char {
    keybinds: KeyBinds,
}

struct KeyBinds {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

#[derive(Default)]
struct CharMotion {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

struct Gravity;

struct Player {
    on_ground: bool,
    size: Vec2,
    velocity: Vec3,
}

struct Terrain {
    size: Vec2,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let char_texture_handle = asset_server.load("textures/char.png");
    let char_texture_atlas =
        TextureAtlas::from_grid(char_texture_handle, Vec2::new(32.0, 32.0), 12, 1);
    let char_texture_atlas_handle = texture_atlases.add(char_texture_atlas);

    let texture_handle = asset_server.load("textures/terrain.png");
    let mut texture_atlas = TextureAtlas::new_empty(texture_handle, Vec2::new(352.0, 176.0));
    for &y in &[0.0f32, 64.0, 128.0] {
        for &x in &[0.0f32, 96.0] {
            texture_atlas.add_texture(bevy::sprite::Rect {
                min: Vec2::new(x + 2.0, y + 1.0),
                max: Vec2::new(x + 48.0 - 2.0, y + 48.0 - 1.0),
            });
        }
    }

    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(2),
            texture_atlas: char_texture_atlas_handle,
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
        .with(Timer::from_seconds(0.1, true))
        .with(Player {
            on_ground: false,
            size: Vec2::new(32.0, 32.0),
            velocity: Vec3::zero(),
        })
        .with(Gravity);

    for x in 0..50 {
        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                    index: 1,
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 44.0 - 300.0,
                    -300.0,
                    0.0,
                )),
                ..Default::default()
            })
            .with(Terrain {
                size: Vec2::new(44.0, 46.0),
            });
    }
}

fn track_inputs_system(
    mut state: ResMut<TrackInputState>,
    keys: Res<Events<KeyboardInput>>,
    cursor: Res<Events<CursorMoved>>,
    motion: Res<Events<MouseMotion>>,
    mousebtn: Res<Events<MouseButtonInput>>,
    scroll: Res<Events<MouseWheel>>,
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

fn animate_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta_seconds);
        if timer.finished {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

fn move_char_system(time: Res<Time>, mut query: Query<(&mut Player, &CharMotion, &mut Transform)>) {
    for (mut player, state, mut transform) in query.iter_mut() {
        if state.up && player.on_ground {
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

fn gravity_system(time: Res<Time>, mut query: Query<(&mut Player)>) {
    for mut player in query.iter_mut() {
        if !player.on_ground {
            player.velocity.y -= 9.8;
        }
    }
}

fn adjust_player_y(
    time: &Res<Time>,
    player: &Player,
    player_transform: &Transform,
    terrain: &Terrain,
    terrain_transform: &Transform,
) -> (Transform, Vec3, bool) {
    let vel = time.delta_seconds * player.velocity;

    let mut new_player_transform = player_transform.clone();
    let mut new_player_velocity = player.velocity.clone();
    let mut on_ground = player.on_ground;

    if player.velocity.y < 0.0 {
        let ty = terrain_transform.translation.y + terrain.size.y;
        let py = player_transform.translation.y + vel.y;
        if py < ty {
            new_player_transform.translation.y = ty;
            new_player_velocity.y = 0.0;
            on_ground = true;
        }
    } else if player.velocity.y > 0.0 {
        let ty = terrain_transform.translation.y;
        let py = player_transform.translation.y + player.size.y + vel.y;
        if py > ty {
            new_player_transform.translation.y = ty;
            new_player_velocity.y = 0.0;
        }
    }

    (new_player_transform, new_player_velocity, on_ground)
}

fn adjust_player_x(
    time: &Res<Time>,
    player: &Player,
    player_transform: &Transform,
    terrain: &Terrain,
    terrain_transform: &Transform,
) -> (Transform, Vec3) {
    let vel = time.delta_seconds * player.velocity;

    let mut new_player_transform = player_transform.clone();
    let mut new_player_velocity = player.velocity.clone();

    if player.velocity.x < 0.0 {
        let tx = terrain_transform.translation.x + terrain.size.x;
        let px = player_transform.translation.x + vel.x;
        if px < tx {
            new_player_transform.translation.x = tx;
            new_player_velocity.x = 0.0;
        }
    } else if player.velocity.x > 0.0 {
        let tx = terrain_transform.translation.x;
        let px = player_transform.translation.x + player.size.x + vel.x;
        if px > tx {
            new_player_transform.translation.x = tx - player.size.x;
            new_player_velocity.x = 0.0;
        }
    }

    (new_player_transform, new_player_velocity)
}

fn physics_system(
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Transform)>,
    mut terrains: Query<(&Terrain, &Transform)>,
) {
    use bevy::render::renderer::RenderResource;

    for (mut player, mut player_transform) in query.iter_mut() {
        let player_translation = player_transform.translation.clone();

        for (terrain, terrain_transform) in terrains.iter_mut() {
            let collision = collide(
                player_transform.translation + time.delta_seconds * player.velocity,
                player.size,
                terrain_transform.translation,
                terrain.size,
            );
            if collision.is_none() {
                continue;
            }

            let (axt, axv) = adjust_player_x(
                &time,
                &player,
                &player_transform,
                &terrain,
                &terrain_transform,
            );
            let (ayt, ayv, on_ground) = adjust_player_y(
                &time,
                &player,
                &player_transform,
                &terrain,
                &terrain_transform,
            );

            let xcol = collide(
                axt.translation + axv,
                player.size,
                terrain_transform.translation,
                terrain.size,
            );
            let ycol = collide(
                ayt.translation + ayv,
                player.size,
                terrain_transform.translation,
                terrain.size,
            );

            match (xcol.is_none(), ycol.is_none()) {
                (_, true) => {
                    *player_transform = ayt;
                    player.velocity = ayv;
                    player.on_ground = on_ground;
                }
                (true, _) => {
                    *player_transform = axt;
                    player.velocity = axv;
                }
                _ => {
                    *player_transform = ayt;
                    player.velocity = ayv;
                    player.on_ground = on_ground;
                    let (axt, axv) = adjust_player_x(
                        &time,
                        &player,
                        &player_transform,
                        &terrain,
                        &terrain_transform,
                    );
                    *player_transform = axt;
                    player.velocity = axv;
                }
            }
        }

        player_transform.translation.x += time.delta_seconds * player.velocity.x;
        player_transform.translation.y += time.delta_seconds * player.velocity.y;
    }
}
