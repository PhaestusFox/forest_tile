use bevy::{
    asset::{RenderAssetUsages, load_internal_asset},
    prelude::*,
    render::render_resource::Extent3d,
    sprite::Material2dPlugin,
    window::{PrimaryWindow, WindowResized},
};

mod shader;

use noise::NoiseFn;
pub use shader::TileMaterial;

pub struct InfiniTilePlugin;

impl Plugin for InfiniTilePlugin {
    fn build(&self, app: &mut App) {
        // load_internal_asset!(
        //     app,
        //     shader::SHADER,
        //     "../assets/infinit_tile.wgsl",
        //     Shader::from_wgsl
        // );
        app.init_resource::<NoiseGenerator>();
        app.add_plugins(Material2dPlugin::<shader::TileMaterial>::default());
        app.add_systems(Startup, spawn_screen_rect)
            .add_systems(PreUpdate, (on_image_load, on_resize, update_noise_map))
            .add_observer(load_noise_map);
    }
}

fn spawn_screen_rect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<shader::TileMaterial>>,
    asset_server: Res<AssetServer>,
    default_material: Option<Res<shader::TileMaterial>>,
) {
    commands.spawn((
        Name::new("Infinite Tile Screen"),
        Mesh2d(meshes.add(shader::screen_mesh())),
        MeshMaterial2d(materials.add(if let Some(material) = default_material {
            material.clone()
        } else {
            shader::TileMaterial {
                texture: asset_server.load("tiles/default.png"),
                noise: Handle::default(),
            }
        })),
    ));
}

#[derive(Resource)]
struct NoiseGenerator {
    noise: noise::SuperSimplex,
}

impl NoiseGenerator {
    fn update_image(&self, image: &mut Image, offset: Vec2) {
        let size = image.size();
        for y in 0..size.y as usize {
            for x in 0..size.x as usize {
                let nx = x as f32 + offset.x;
                let ny = y as f32 + offset.y;
                let value = self.noise.get([nx as f64, ny as f64]) as f32;
                let color = Color::linear_rgb(value, value, value);
                if let Err(e) = image.set_color_at(x as u32, y as u32, color) {
                    warn!("Failed to set color at ({}, {}): {}", x, y, e);
                };
            }
        }
    }
}

impl FromWorld for NoiseGenerator {
    fn from_world(_world: &mut World) -> Self {
        NoiseGenerator {
            noise: noise::SuperSimplex::new(0),
        }
    }
}

fn update_noise_map(
    noise: Res<NoiseGenerator>,
    materials: Res<Assets<shader::TileMaterial>>,
    query: Query<&MeshMaterial2d<shader::TileMaterial>>,
    camera: Single<&Transform, With<Camera2d>>,
    mut images: ResMut<Assets<Image>>,
) {
    let offset = camera.translation.truncate();
    for material in query.iter() {
        let Some(material) = materials.get(&material.0) else {
            warn!("Material not found for noise update");
            continue;
        };

        let Some(image) = images.get_mut(&material.noise) else {
            warn!("Noise Texture not found for material");
            continue;
        };
        noise.update_image(image, offset);
    }
}

fn load_noise_map(
    event: Trigger<GenNewNoiseImage>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TileMaterial>>,
) {
    info!(
        "Generating new noise image for screen size: {:?}",
        event.event().screen_size
    );

    let GenNewNoiseImage {
        screen_size,
        handle,
    } = event.event();
    let Some(material) = materials.get_mut(handle) else {
        warn!("Material handle not found for noise generation");
        return;
    };
    let Some(tile) = images.get(&material.texture) else {
        warn!("Tile texture handle not loaded for noise generation");
        return;
    };
    let IVec2 {
        x: x_tiles,
        y: y_tiles,
    } = (screen_size / tile.size_f32()).ceil().as_ivec2();

    if material.noise == Handle::default() {
        material.noise = images.add(Image::new(
            Extent3d {
                width: x_tiles as u32,
                height: y_tiles as u32,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            vec![0; (x_tiles * y_tiles * 4) as usize],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        ));
        return;
    }

    let Some(image) = images.get_mut(&material.noise) else {
        warn!("Noise texture not found after creation");
        return;
    };

    if image.size() != UVec2::new(x_tiles as u32, y_tiles as u32) {
        image.resize(Extent3d {
            width: x_tiles as u32,
            height: y_tiles as u32,
            depth_or_array_layers: 1,
        });
    }
}

#[derive(Event)]
struct GenNewNoiseImage {
    screen_size: Vec2,
    handle: Handle<TileMaterial>,
}

fn on_resize(
    mut event: EventReader<WindowResized>,
    mut commands: Commands,
    materials: Query<&MeshMaterial2d<TileMaterial>>,
) {
    let Some(event) = event.read().last() else {
        return;
    };
    for material in materials.iter() {
        let handle = material.0.clone();
        commands.trigger(GenNewNoiseImage {
            screen_size: Vec2::new(event.width, event.height),
            handle: handle.clone_weak(),
        });
    }
}

fn on_image_load(
    mut events: EventReader<AssetEvent<Image>>,
    mut commands: Commands,
    chunks: Query<&MeshMaterial2d<shader::TileMaterial>>,
    materials: Res<Assets<TileMaterial>>,
    mut loaded: Local<bevy::platform::collections::HashSet<AssetId<Image>>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                loaded.insert(*id);
            }
            _ => continue,
        }
    }
    if loaded.is_empty() {
        return;
    }
    for chunk in chunks.iter() {
        if let Some(material) = materials.get(&chunk.0)
            && loaded.contains(&material.texture.id())
        {
            commands.trigger(GenNewNoiseImage {
                screen_size: Vec2::new(window.width(), window.height()),
                handle: chunk.0.clone_weak(),
            });
        }
    }
    loaded.clear();
}
