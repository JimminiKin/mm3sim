use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

use crate::resources::constants::*;

#[derive(Component)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: CAMERA_INITIAL_RADIUS,
            yaw: CAMERA_INITIAL_YAW,
            pitch: CAMERA_INITIAL_PITCH,
        }
    }
}

pub fn orbit_camera_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let mut delta = Vec2::ZERO;
    let mut zoom = 0.0;

    if mouse_buttons.pressed(MouseButton::Right) {
        for ev in motion.read() {
            delta += ev.delta;
        }
    } else {
        motion.clear();
    }

    for ev in scroll.read() {
        zoom += match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y * CAMERA_SCROLL_PIXEL_FACTOR,
        };
    }

    let Ok((mut cam, mut transform)) = query.get_single_mut() else { return };

    cam.yaw -= delta.x * CAMERA_ORBIT_SENSITIVITY;
    cam.pitch = (cam.pitch - delta.y * CAMERA_ORBIT_SENSITIVITY)
        .clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);
    cam.radius = (cam.radius - zoom * CAMERA_ZOOM_SPEED)
        .clamp(CAMERA_RADIUS_MIN, CAMERA_RADIUS_MAX);

    let rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
    transform.translation = cam.focus + rotation * Vec3::new(0.0, 0.0, cam.radius);
    transform.look_at(cam.focus, Vec3::Y);
}
