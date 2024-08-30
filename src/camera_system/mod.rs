mod mouse;

use bevy::prelude::*;
pub use mouse::{orbit_mouse, MousePlugin};

use crate::loading_screen;

pub struct ThirdPersonCameraPlugin;

impl Plugin for ThirdPersonCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MousePlugin).add_systems(
            Update,
            (sync_player_camera
                .after(orbit_mouse)
                .run_if(in_state(loading_screen::AppState::InGame)),),
        );
    }
}

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub focus: Vec3,
    pub mouse_sensitivity: f32,
    pub mouse_orbit_button: MouseButton,
    pub zoom_enabled: bool,
    pub zoom: Zoom,
    pub zoom_sensitivity: f32,
    pub inertia: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        ThirdPersonCamera {
            focus: Vec3::ZERO,
            mouse_sensitivity: 2.5,
            mouse_orbit_button: MouseButton::Right,
            zoom_enabled: true,
            zoom: Zoom::new(1.5, 9.0),
            zoom_sensitivity: 1.0,
            inertia: 0.97,
        }
    }
}

pub struct Zoom {
    pub min: f32,
    pub max: f32,
    radius: f32,
}

impl Zoom {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            radius: (min + max) / 2.0,
        }
    }
}

#[derive(Component)]
pub struct ThirdPersonCameraTarget;

fn sync_player_camera(
    player_q: Query<&Transform, With<ThirdPersonCameraTarget>>,
    mut cam_q: Query<(&mut ThirdPersonCamera, &mut Transform), Without<ThirdPersonCameraTarget>>,
) {
    let Ok(player) = player_q.get_single() else {
        return;
    };
    let Ok((cam, mut cam_transform)) = cam_q.get_single_mut() else {
        return;
    };
    let rotation_matrix = Mat3::from_quat(cam_transform.rotation);

    let desired_translation =
        cam.focus + rotation_matrix.mul_vec3(Vec3::new(0.0, 0.0, cam.zoom.radius));

    let delta = player.translation - cam.focus;
    cam_transform.translation = desired_translation + delta;
}

pub fn zoom_condition(cam_q: Query<&ThirdPersonCamera, With<ThirdPersonCamera>>) -> bool {
    let Ok(cam) = cam_q.get_single() else {
        return false;
    };
    return cam.zoom_enabled;
}
