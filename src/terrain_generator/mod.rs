use bevy::{prelude::*, render::render_resource::AsBindGroup};

mod material;
mod mesh_generator;
mod noise_generator;
use noise::utils::NoiseMap;

use crate::config_parser::*;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {}

#[derive(Resource)]
pub struct TerrainMap {
    pub map: NoiseMap,
}

pub async fn create_texture_map(map_config: MapConfig, engine_config: EngineConfig) -> NoiseMap {
    let map: NoiseMap = noise_generator::generate_texture(&map_config, &engine_config);
    return map;
}

pub async fn create_map_mesh(
    engine_config: EngineConfig,
    map: Vec<f64>,
) -> (Vec<Mesh>, Vec<(Vec<Vec3>, Vec<[u32; 3]>)>) {
    let (meshes, colliders) = mesh_generator::generate_low_poly_terrain(engine_config, map);
    return (meshes, colliders);
}
