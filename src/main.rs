use bevy::{asset::AssetServerSettings, prelude::*};

fn main() {
    App::build()
        .add_resource(AssetServerSettings {
            asset_folder: option_env!("MCP2_PREFIX").unwrap_or("/").to_string(),
        })
        .add_plugins(bevy_webgl2::DefaultPlugins)
        .add_startup_system(load_asset)
        .run();
}

fn load_asset(
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
