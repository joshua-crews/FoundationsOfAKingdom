use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

use crate::config_parser::EngineConfig;

pub fn generate_low_poly_terrain(
    engine_config: EngineConfig,
    map: Vec<f64>,
) -> (Vec<Mesh>, Vec<(Vec<Vec3>, Vec<[u32; 3]>)>) {
    validate_engine_config(&engine_config);
    let mut meshes = Vec::new();
    let mut colliders = Vec::new();
    let flattened_map: Vec<Vec<f64>> = map
        .chunks(engine_config.world_size as usize)
        .map(|chunk| chunk.to_vec())
        .collect();
    let chunk_ratio = (engine_config.world_size / engine_config.chunk_size) + 1;

    for i in 0..chunk_ratio {
        for j in 0..chunk_ratio {
            let mut collider_indices = compute_collider_indices(&engine_config);
            let mut collider_vertices = compute_collider_vertices(&engine_config, &map, &i, &j);

            let mut indices = Vec::new();
            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            let mut colors = Vec::new();

            let x_start = (i * engine_config.chunk_size).saturating_sub(i) as usize;
            let z_start = (j * engine_config.chunk_size).saturating_sub(j) as usize;
            let x_end = x_start + engine_config.chunk_size as usize;
            let z_end = z_start + engine_config.chunk_size as usize;

            for z in z_start..z_end - 1 {
                for x in x_start..x_end - 1 {
                    let y_top_left = flattened_map[z][x] as f32 * engine_config.world_height;
                    let y_top_right = flattened_map[z][x + 1] as f32 * engine_config.world_height;
                    let y_bottom_left = flattened_map[z + 1][x] as f32 * engine_config.world_height;
                    let y_bottom_right =
                        flattened_map[z + 1][x + 1] as f32 * engine_config.world_height;

                    // Define vertices for the first triangle (top-left, bottom-left, bottom-right)
                    let base_index = vertices.len() as u32;
                    vertices.push([
                        x as f32 - x_start as f32,
                        y_top_left,
                        z as f32 - z_start as f32,
                    ]);
                    vertices.push([
                        x as f32 - x_start as f32,
                        y_bottom_left,
                        (z + 1) as f32 - z_start as f32,
                    ]);
                    vertices.push([
                        (x + 1) as f32 - x_start as f32,
                        y_bottom_right,
                        (z + 1) as f32 - z_start as f32,
                    ]);

                    indices.extend_from_slice(&[base_index, base_index + 1, base_index + 2]);
                    let avg_y1 = (y_top_left + y_bottom_left + y_bottom_right) / 3.0;
                    let triangle_color1: [f32; 4] = match avg_y1 {
                        avg_y1 if avg_y1 < -0.3 => [0.829, 0.806, 0.567, 1.0],
                        avg_y1 if avg_y1 >= -0.3 && avg_y1 < 0.5 => [0.625, 0.96, 0.559, 1.0],
                        _ => [0.5, 0.5, 0.5, 1.0],
                    };
                    for _i in 0..3 {
                        colors.push(triangle_color1);
                    }

                    let normal1 =
                        calculate_normal(&vertices, [base_index, base_index + 1, base_index + 2]);
                    normals.extend_from_slice(&[normal1, normal1, normal1]);

                    // Define vertices for the second triangle (top-left, bottom-right, top-right)
                    vertices.push([
                        x as f32 - x_start as f32,
                        y_top_left,
                        z as f32 - z_start as f32,
                    ]);
                    vertices.push([
                        (x + 1) as f32 - x_start as f32,
                        y_bottom_right,
                        (z + 1) as f32 - z_start as f32,
                    ]);
                    vertices.push([
                        (x + 1) as f32 - x_start as f32,
                        y_top_right,
                        z as f32 - z_start as f32,
                    ]);

                    indices.extend_from_slice(&[base_index + 3, base_index + 4, base_index + 5]);
                    let avg_y2 = (y_top_left + y_bottom_right + y_top_right) / 3.0;
                    let triangle_color2: [f32; 4] = match avg_y2 {
                        avg_y2 if avg_y2 < -0.3 => [0.829, 0.806, 0.567, 1.0],
                        avg_y2 if avg_y2 >= -0.3 && avg_y2 < 0.5 => [0.625, 0.96, 0.559, 1.0],
                        _ => [0.5, 0.5, 0.5, 1.0],
                    };
                    for _i in 0..3 {
                        colors.push(triangle_color2);
                    }

                    let normal2 = calculate_normal(
                        &vertices,
                        [base_index + 3, base_index + 4, base_index + 5],
                    );
                    normals.extend_from_slice(&[normal2, normal2, normal2]);
                }
            }

            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            mesh.insert_indices(Indices::U32(indices.clone()));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());

            meshes.push(mesh);
            colliders.push((collider_vertices, collider_indices));
        }
    }

    return (meshes, colliders);
}

fn compute_collider_vertices(
    engine_config: &EngineConfig,
    map: &Vec<f64>,
    i: &usize,
    j: &usize,
) -> Vec<Vec3> {
    let mut vertices = Vec::new();
    for z in 0..engine_config.chunk_size {
        for x in 0..engine_config.chunk_size {
            let index = (((j * (engine_config.world_size / engine_config.chunk_size)) + x)
                + ((i * (engine_config.world_size / engine_config.chunk_size)) + z)
                    * engine_config.world_size)
                - (i * engine_config.world_size)
                - j;
            let y = map[index] as f32 * engine_config.world_height;
            vertices.push([x as f32, y, z as f32]);
        }
    }
    let collider_vertices = vertices.into_iter().map(Vec3::from).collect();
    return collider_vertices;
}

fn compute_collider_indices(engine_config: &EngineConfig) -> Vec<[u32; 3]> {
    let mut indices = Vec::new();
    for y in 0..(engine_config.chunk_size - 1) {
        for x in 0..(engine_config.chunk_size - 1) {
            let top_left = y * engine_config.chunk_size + x;
            let top_right = y * engine_config.chunk_size + x + 1;
            let bottom_left = (y + 1) * engine_config.chunk_size + x;
            let bottom_right = (y + 1) * engine_config.chunk_size + x + 1;

            indices.push(top_left as u32);
            indices.push(bottom_left as u32);
            indices.push(top_right as u32);

            indices.push(top_right as u32);
            indices.push(bottom_left as u32);
            indices.push(bottom_right as u32);
        }
    }
    let collider_indices = indices
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect::<Vec<[u32; 3]>>();
    return collider_indices;
}

fn validate_engine_config(engine_config: &EngineConfig) {
    if engine_config.world_size % engine_config.chunk_size != 0 {
        panic!(
            "Chunk size {:?} must be a factor of world size {:?}",
            engine_config.chunk_size, engine_config.world_size
        );
    }
}

fn calculate_normal(vertices: &Vec<[f32; 3]>, indices: [u32; 3]) -> [f32; 3] {
    let v0 = vertices[indices[0] as usize];
    let v1 = vertices[indices[1] as usize];
    let v2 = vertices[indices[2] as usize];

    let u = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let v = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let normal = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];

    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if length != 0.0 {
        [normal[0] / length, normal[1] / length, normal[2] / length]
    } else {
        normal
    }
}
