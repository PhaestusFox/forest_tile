use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, infini_tile::InfiniTilePlugin))
        .add_systems(PreStartup, set_default_material)
        .add_systems(Startup, (setup, spawn_test_square))
        .add_systems(Update, pan_camera)
        .run();
}

fn set_default_material(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(infini_tile::TileMaterial {
        texture: asset_server.load("no_share/minecraft/textures/block/grass_block_top.png"),
        noise: Handle::default(),
    });
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_test_square(mut commands: Commands) {
    commands.spawn((
        Name::new("Test Square"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(100.0, 100.0)),
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
    camera_query.translation += delta * time.delta_secs() * 100.0;
}
