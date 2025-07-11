use bevy::{
    asset::{RenderAssetUsages, weak_handle},
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite::Material2d,
};

// pub(crate) const SHADER: Handle<Shader> = weak_handle!("38bef9da-a715-4a3c-9bdb-1a62b0e52621");
pub(crate) const SHADER: &str = "infinit_tile.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Clone, Resource)]
pub struct TileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[texture(2, dimension = "2d_array")]
    pub noise: Handle<Image>,
    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub decor_array: Handle<Image>,
}

impl Material2d for TileMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SHADER.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Mask(0.5)
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub(crate) fn screen_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::all(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0., 0.], [1., 0.], [0., 1.], [1., 1.]],
    );

    mesh.insert_indices(bevy::render::mesh::Indices::U16(vec![0, 1, 2, 3]));

    mesh
}
