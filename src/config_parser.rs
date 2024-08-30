use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Clone, Debug, Serialize, Deserialize, Resource)]
pub struct MapConfig {
    pub seed: u32,
    pub continent_frequency: f64,
    pub continent_lacunarity: f64,
    pub mountain_lacunarity: f64,
    pub hills_lacunarity: f64,
    pub plains_lacunarity: f64,
    pub badlands_lacunarity: f64,
    pub mountains_twist: f64,
    pub hills_twist: f64,
    pub badlands_twist: f64,
    pub sea_level: f64,
    pub shelf_level: f64,
    pub mountains_amount: f64,
    pub hills_amount: f64,
    pub badlands_amount: f64,
    pub terrain_offset: f64,
    pub mountain_glaciation: f64,
    pub continent_height_scale: f64,
    pub river_depth: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Resource)]
pub struct EngineConfig {
    pub world_size: usize,
    pub chunk_size: usize,
    pub world_height: f32,
}

pub fn read_configs(mut commands: Commands) {
    // Map config
    let map_file = std::fs::File::open("assets/configs/map_generation.yml")
        .expect("Could not open map config file.");
    let map_config: MapConfig =
        serde_yaml::from_reader(map_file).expect("Could not read map settings.");
    commands.insert_resource(map_config);

    // Engine config
    let engine_file = std::fs::File::open("assets/configs/engine_config.yml")
        .expect("Could not open engine config file.");
    let engine_config: EngineConfig =
        serde_yaml::from_reader(engine_file).expect("Could not read engine settings.");
    commands.insert_resource(engine_config);
}
