use std::f32::consts::PI;

use bevy::{
    pbr::{
        wireframe::Wireframe, CascadeShadowConfigBuilder, ExtendedMaterial, OpaqueRendererMethod,
    },
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};

use bevy_rapier3d::prelude::*;

use crate::camera_system;
use crate::config_parser;
use crate::terrain_generator;

use iyes_perf_ui::prelude::PerfUiCompleteBundle;

use bevy_asset_loader::prelude::*;
use futures_lite::future;
use noise::utils::NoiseMap;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    LoadingConfigs,
    GeneratingTerrain,
    GeneratingMeshes,
    InGame,
}

#[derive(Component)]
struct LoadingScreenComponent;

#[derive(Component)]
struct ComputeMapComponent(Task<NoiseMap>);

#[derive(Component)]
struct ComputeMeshComponent(Task<(Vec<Mesh>, Vec<(Vec<Vec3>, Vec<[u32; 3]>)>)>);

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_loading_state(
                LoadingState::new(AppState::LoadingConfigs)
                    .continue_to_state(AppState::GeneratingTerrain),
            )
            .add_systems(OnEnter(AppState::LoadingConfigs), loading_screen)
            .add_systems(
                OnEnter(AppState::LoadingConfigs),
                config_parser::read_configs,
            )
            .add_systems(OnEnter(AppState::GeneratingTerrain), generate_terrain)
            .add_systems(
                Update,
                handle_map_generation_tasks.run_if(in_state(AppState::GeneratingTerrain)),
            )
            .add_systems(OnEnter(AppState::GeneratingMeshes), mesh_terrain)
            .add_systems(
                Update,
                handle_map_mesh_tasks.run_if(in_state(AppState::GeneratingMeshes)),
            )
            .add_systems(OnEnter(AppState::InGame), enter_game)
            .add_systems(Update, player_movement.run_if(in_state(AppState::InGame)));
    }
}

fn loading_screen(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), LoadingScreenComponent));
    commands.spawn((
        TextBundle::from_section(
            "Loading...",
            TextStyle {
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center),
        LoadingScreenComponent,
    ));
    commands.spawn(PerfUiCompleteBundle::default());
}

fn generate_terrain(
    mut commands: Commands,
    map_config: Res<config_parser::MapConfig>,
    engine_config: Res<config_parser::EngineConfig>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let m_config = map_config.clone();
    let e_config = engine_config.clone();
    let task = thread_pool.spawn(async move {
        let map = terrain_generator::create_texture_map(m_config, e_config).await;
        return map;
    });
    commands.spawn(()).insert(ComputeMapComponent(task));
}

fn handle_map_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ComputeMapComponent)>,
    mut state: ResMut<NextState<AppState>>,
) {
    for (entity, mut task_component) in tasks.iter_mut() {
        let future = future::block_on(future::poll_once(&mut task_component.0));
        if let Some(map_data) = future {
            commands.insert_resource(terrain_generator::TerrainMap { map: map_data });
            commands.entity(entity).remove::<ComputeMapComponent>();
            info!(
                target: "Foundations_Of_A_Kingdom::loading_state::systems",
                "Loading state 'Foundations_Of_A_Kingdom::loading_screen::AppState::GeneratingTerrain' is done"
            );
            state.set(AppState::GeneratingMeshes);
        }
    }
}

fn mesh_terrain(
    mut commands: Commands,
    engine_config: Res<config_parser::EngineConfig>,
    height_map: Res<terrain_generator::TerrainMap>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let e_config = engine_config.clone();
    let map = height_map.map.iter().copied().collect();
    let task = thread_pool.spawn(async move {
        let (meshes, colliders) = terrain_generator::create_map_mesh(e_config, map).await;
        return (meshes, colliders);
    });
    commands.spawn(()).insert(ComputeMeshComponent(task));
}

fn handle_map_mesh_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ComputeMeshComponent)>,
    mut state: ResMut<NextState<AppState>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    /*
    mut materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, terrain_generator::TerrainMaterial>>,
    >,
    */
    mut meshes: ResMut<Assets<Mesh>>,
    engine_config: Res<config_parser::EngineConfig>,
) {
    for (entity, mut task_component) in tasks.iter_mut() {
        let future = future::block_on(future::poll_once(&mut task_component.0));
        if let Some((map_meshes, colliders)) = future {
            let scale_factor = (engine_config.world_size / engine_config.chunk_size) + 1;
            for x in 0..scale_factor {
                for z in 0..scale_factor {
                    let index = (x * scale_factor) + z;
                    commands.spawn((
                        MaterialMeshBundle {
                            mesh: meshes.add(map_meshes[index].clone()),
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb_u8(255, 255, 255),
                                opaque_render_method: OpaqueRendererMethod::Auto,
                                metallic: 0.0,
                                reflectance: 0.0,
                                perceptual_roughness: 1.0,
                                ..Default::default()
                            }),
                            /*
                            material: materials.add(ExtendedMaterial {
                                base: StandardMaterial {
                                    base_color: Color::srgb_u8(124, 144, 255),
                                    opaque_render_method: OpaqueRendererMethod::Auto,
                                    metallic: 0.0,
                                    reflectance: 0.0,
                                    perceptual_roughness: 1.0,
                                    ..Default::default()
                                },
                                extension: terrain_generator::TerrainMaterial {},
                            }),
                            */
                            transform: Transform::from_xyz(
                                x as f32 * (engine_config.chunk_size as f32 - 1.0),
                                0.0,
                                z as f32 * (engine_config.chunk_size as f32 - 1.0),
                            ),
                            ..default()
                        },
                        //Collider::trimesh(colliders[index].0.clone(), colliders[index].1.clone()),
                        //Wireframe,
                    ));
                }
            }
            commands.entity(entity).remove::<ComputeMeshComponent>();
            info!(
                target: "Foundations_Of_A_Kingdom::loading_state::systems",
                "Loading state 'Foundations_Of_A_Kingdom::loading_screen::AppState::GeneratingMeshes' is done"
            );
            state.set(AppState::InGame);
        }
    }
}

fn enter_game(
    mut commands: Commands,
    loading_query: Query<Entity, With<LoadingScreenComponent>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for loading_component in loading_query.iter() {
        commands.entity(loading_component).despawn();
    }
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 30.0,
            ..default()
        }
        .into(),
        ..default()
    });
    commands.spawn((
        camera_system::ThirdPersonCamera::default(),
        Camera3dBundle::default(),
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(244, 90, 90)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        camera_system::ThirdPersonCameraTarget,
    ));
}

fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<&mut Transform, With<camera_system::ThirdPersonCameraTarget>>,
    cam_q: Query<
        &Transform,
        (
            With<Camera3d>,
            Without<camera_system::ThirdPersonCameraTarget>,
        ),
    >,
) {
    for mut player_transform in player_q.iter_mut() {
        let cam = match cam_q.get_single() {
            Ok(c) => c,
            Err(e) => Err(format!("Error retrieving camera: {}", e)).unwrap(),
        };

        let mut direction = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            direction += *cam.forward();
        }
        if keys.pressed(KeyCode::KeyS) {
            direction += *cam.back();
        }
        if keys.pressed(KeyCode::KeyA) {
            direction += *cam.left();
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += *cam.right();
        }
        direction.y = 0.0;
        let movement = direction.normalize_or_zero() * 4.5 * time.delta_seconds();
        player_transform.translation += movement;
        if direction.length_squared() > 0.0 {
            player_transform.look_to(direction, Vec3::Y);
        }
    }
}
