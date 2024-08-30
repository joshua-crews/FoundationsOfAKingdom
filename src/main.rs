mod camera_system;
mod config_parser;
mod loading_screen;
mod terrain_generator;

use bevy::{
    pbr::{wireframe::WireframePlugin, ExtendedMaterial},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use camera_system::ThirdPersonCameraPlugin;
use iyes_perf_ui::prelude::*;
use terrain_generator::TerrainMaterial;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1280.0, 720.0).into(),
                    title: "Foundations of a Kingdom".into(),
                    ..default()
                }),
                ..default()
            }),
            //Physics
            RapierPhysicsPlugin::<NoUserData>::default(),
            // Debug Systems
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            PerfUiPlugin,
            WireframePlugin,
            RapierDebugRenderPlugin::default(),
            //Game logic
            loading_screen::LoadingScreenPlugin,
            ThirdPersonCameraPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, TerrainMaterial>>::default(),
        ))
        .run();
}
