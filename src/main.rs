use bevy::{
    asset::AssetServerSettings,
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    prelude::*,
    render::camera::Camera,
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
        .add_system(camera_system)
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

    for y in 0..5 {
        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                    index: 2,
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    100.0,
                    y as f32 * 44.0 - 100.0,
                    0.0,
                )),
                ..Default::default()
            })
            .with(Terrain {
                size: Vec2::new(44.0, 46.0),
            });
    }

    for y in 0..5 {
        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                    index: 2,
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    -200.0,
                    y as f32 * 44.0 - 300.0,
                    0.0,
                )),
                ..Default::default()
            })
            .with(Terrain {
                size: Vec2::new(44.0, 46.0),
            });
    }

    for x in 0..5 {
        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                    index: 1,
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 44.0 - 300.0,
                    -100.0,
                    0.0,
                )),
                ..Default::default()
            })
            .with(Terrain {
                size: Vec2::new(44.0, 46.0),
            });
    }

    for x in (0..50).filter(|&x| x < 15 || x > 20) {
        commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                    index: 1,
                },
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 44.0 - 500.0,
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

fn move_char_system(mut query: Query<(&mut Player, &CharMotion)>) {
    for (mut player, state) in query.iter_mut() {
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

fn gravity_system(mut query: Query<&mut Player>) {
    for mut player in query.iter_mut() {
        player.velocity.y -= 9.8;
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
