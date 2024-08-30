use crate::terrain_generator;
use bevy::{pbr::MaterialExtension, render::render_resource::ShaderRef};

impl MaterialExtension for terrain_generator::TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}
