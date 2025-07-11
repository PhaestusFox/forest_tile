use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, infini_tile::InfiniTilePlugin, decor::plugin))
        .add_systems(PreStartup, set_default_material)
        .add_systems(Startup, setup)
        .add_systems(Update, (pan_camera, step_camera))
        .run();
}

mod decor;

fn set_default_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loading: Res<decor::LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(infini_tile::TileMaterial {
        texture: asset_server.load("no_share/minecraft/textures/block/grass_block_top.png"),
        noise: images.add(Image::new(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 4,
            },
            TextureDimension::D2,
            vec![0; 4 * 4],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        )),
        decor_array: loading.get_handle(),
    });
}

fn setup(mut commands: Commands) {
    commands
        .spawn((Camera2d, Transform::from_translation(Vec3::NEG_Y)))
        .with_child((
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
        ));
}
fn pan_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Single<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let mut delta = Vec3::ZERO;
    if input.pressed(KeyCode::KeyA) {
        delta.x -= 1.0;
    } else if input.pressed(KeyCode::KeyD) {
        delta.x += 1.0;
    }
    if input.pressed(KeyCode::KeyW) {
        delta.y += 1.0;
    } else if input.pressed(KeyCode::KeyS) {
        delta.y -= 1.0;
    }
    camera_query.translation += delta * time.delta_secs() * 1000.0;
}

fn step_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Single<&mut Transform, With<Camera2d>>,
) {
    let mut moved = false;
    for key in input.get_just_pressed() {
        match key {
            KeyCode::Numpad0 => {
                camera_query.translation = Vec3::new(0.0, 0.0, 0.0);
                moved = true;
            }
            KeyCode::Numpad1 => {
                camera_query.translation.x -= 1.0;
                moved = true;
            }
            KeyCode::Numpad4 => {
                camera_query.translation.x -= 5.0;
                moved = true;
            }
            KeyCode::Numpad7 => {
                camera_query.translation.x -= 10.0;
                moved = true;
            }
            KeyCode::Numpad9 => {
                camera_query.translation.x += 10.0;
                moved = true;
            }
            KeyCode::Numpad6 => {
                camera_query.translation.x += 5.0;
                moved = true;
            }
            KeyCode::Numpad3 => {
                camera_query.translation.x += 1.0;
                moved = true;
            }
            KeyCode::Numpad2 => {
                camera_query.translation.y -= 1.0;
                moved = true;
            }
            KeyCode::Numpad8 => {
                camera_query.translation.y += 1.0;
                moved = true;
            }
            _ => {}
        }
    }
    if moved {
        info!("Camera moved to: {:?}", camera_query.translation);
    }
}
