use std::f32::consts::PI;

use bevy::ecs::query::QuerySingleError;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use crate::camera_system::ThirdPersonCamera;
use crate::{camera_system, loading_screen::AppState::InGame};

#[derive(Resource)]
pub struct CamVelocity(Vec2);
pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CamVelocity(Vec2::ZERO)).add_systems(
            Update,
            (
                orbit_mouse.run_if(in_state(InGame)),
                zoom_mouse
                    .run_if(in_state(InGame))
                    .run_if(camera_system::zoom_condition),
            )
                .chain(),
        );
    }
}

fn orbit_condition(cam: &ThirdPersonCamera, mouse: &Res<ButtonInput<MouseButton>>) -> bool {
    if mouse.pressed(cam.mouse_orbit_button) {
        return true;
    }
    return false;
}

pub fn orbit_mouse(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cam_q: Query<(&ThirdPersonCamera, &mut Transform), With<ThirdPersonCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_evr: EventReader<MouseMotion>,
    mut cam_velocity: ResMut<CamVelocity>,
) {
    let rotation: Vec2;
    let Ok((cam, mut cam_transform)): Result<
        (&ThirdPersonCamera, Mut<Transform>),
        QuerySingleError,
    > = cam_q.get_single_mut() else {
        return;
    };
    let mut position: Vec2 = Vec2::new(0.0, 0.0);
    for ev in mouse_evr.read() {
        if orbit_condition(cam, &mouse) {
            cam_velocity.0 = ev.delta * cam.mouse_sensitivity;
        }
        position = ev.delta * cam.mouse_sensitivity;
    }

    if !orbit_condition(cam, &mouse) {
        rotation = cam_velocity.0;
        cam_velocity.0 *= cam.inertia;
    } else {
        rotation = position;
        cam_velocity.0 = position;
    }

    if rotation.length_squared() > 0.0 {
        let window = window_q.get_single().unwrap();
        let delta_x = {
            let delta: f32 = rotation.x / window.width() * PI;
            delta
        };

        let delta_y: f32 = rotation.y / window.height() * PI;
        let yaw: Quat = Quat::from_rotation_y(-delta_x);
        let pitch: Quat = Quat::from_rotation_x(-delta_y);
        cam_transform.rotation = yaw * cam_transform.rotation;

        let new_rotation: Quat = cam_transform.rotation * pitch;
        let up_vector: Vec3 = new_rotation * Vec3::Y;
        if up_vector.y > 0.0 {
            cam_transform.rotation = new_rotation;
        }
    }

    let rot_matrix: Mat3 = Mat3::from_quat(cam_transform.rotation);
    cam_transform.translation =
        cam.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, cam.zoom.radius));
}

fn zoom_mouse(
    mut scroll_evr: EventReader<MouseWheel>,
    mut cam_q: Query<&mut ThirdPersonCamera>,
    cam_transform_q: Query<&Transform, With<ThirdPersonCamera>>,
) {
    let mut scroll: f32 = 0.0;
    for ev in scroll_evr.read() {
        scroll += ev.y;
    }

    if let (Ok(mut cam), Ok(_camera_transform)) =
        (cam_q.get_single_mut(), cam_transform_q.get_single())
    {
        if scroll.abs() > 0.0 {
            let new_radius: f32 =
                cam.zoom.radius - scroll * cam.zoom.radius * 0.1 * cam.zoom_sensitivity;
            cam.zoom.radius = new_radius.clamp(cam.zoom.min, cam.zoom.max);
        }
    }
}
