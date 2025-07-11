use bevy::prelude::*;

#[derive(Resource)]
pub struct LoadingTexture {
    can_build: bool,
    has_built: bool,
    to_load: Vec<Handle<Image>>,
    load_into: Handle<Image>,
}

impl LoadingTexture {
    pub fn get_handle(&self) -> Handle<Image> {
        self.load_into.clone()
    }
}

impl FromWorld for LoadingTexture {
    fn from_world(world: &mut World) -> Self {
        let target = world.resource_mut::<Assets<Image>>().reserve_handle();
        let asset_server = world.resource::<AssetServer>();
        LoadingTexture {
            can_build: false,
            has_built: false,
            to_load: vec![
                asset_server.load("no_share/minecraft/textures/block/dandelion.png"),
                asset_server.load("no_share/minecraft/textures/block/poppy.png"),
                asset_server.load("no_share/minecraft/textures/block/crimson_fungus.png"),
                asset_server.load("no_share/minecraft/textures/block/wither_rose.png"),
                asset_server.load("no_share/minecraft/textures/block/red_tulip.png"),
                asset_server.load("no_share/minecraft/textures/block/dirt.png"),
                asset_server.load("no_share/minecraft/textures/block/coarse_dirt.png"),
                asset_server.load("no_share/minecraft/textures/block/rooted_dirt.png"),
            ],
            load_into: target,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<LoadingTexture>()
        .add_systems(
            Update,
            build_texture_array.run_if(|to_load: Res<LoadingTexture>| !to_load.has_built),
        )
        .add_systems(
            Update,
            check_if_all_loaded.run_if(|to_load: Res<LoadingTexture>| !to_load.can_build),
        );
}

fn check_if_all_loaded(
    mut loading_texture: ResMut<LoadingTexture>,
    asset_server: Res<AssetServer>,
) {
    for image in loading_texture.to_load.iter() {
        if !asset_server.is_loaded(image) {
            return;
        }
    }
    loading_texture.can_build = true;
}

fn build_texture_array(
    mut textures: ResMut<Assets<Image>>,
    mut loading_texture: ResMut<LoadingTexture>,
) {
    loading_texture.has_built = true;
    if loading_texture.to_load.is_empty() {
        return;
    }
    let Some(template) = textures.get(&loading_texture.to_load[0]) else {
        warn!("First image not loaded for texture array");
        return;
    };
    let mut atlas = TextureAtlasBuilder::default();
    println!("size: {:?}", template.size());
    atlas.max_size(UVec2::new(template.size().x, u32::MAX));
    atlas.initial_size(template.size());
    for image in loading_texture.to_load.iter() {
        if let Some(texture) = textures.get(image) {
            atlas.add_texture(None, texture);
        } else {
            warn!("Image {:?} not loaded for texture array", image);
        }
    }
    println!(
        "Building texture atlas with {} images",
        loading_texture.to_load.len()
    );
    match atlas.build() {
        Ok((_, _, mut image)) => {
            println!("Texture atlas built successfully:{}", image.size());
            image.reinterpret_stacked_2d_as_array(loading_texture.to_load.len() as u32);
            textures.insert(loading_texture.load_into.id(), image);
        }
        Err(e) => {
            error!("Failed to build texture atlas: {:?}", e);
        }
    }
}
