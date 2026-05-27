use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy_egui::EguiContexts;

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
    mut motion: MessageReader<MouseMotion>,
    mut scroll: MessageReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    mut contexts: EguiContexts,
) {
    let ui_wants_scroll = contexts.ctx_mut().expect("primary egui context").is_pointer_over_area();

    let orbiting = mouse_buttons.pressed(MouseButton::Right);
    let panning  = mouse_buttons.pressed(MouseButton::Middle);

    let mut orbit_delta = Vec2::ZERO;
    let mut pan_delta   = Vec2::ZERO;
    let mut zoom        = 0.0;

    if orbiting || panning {
        for ev in motion.read() {
            if orbiting { orbit_delta += ev.delta; }
            if panning  { pan_delta   += ev.delta; }
        }
    } else {
        motion.clear();
    }

    // Always drain scroll events; only apply to zoom when egui is not using them.
    for ev in scroll.read() {
        if !ui_wants_scroll {
            zoom += match ev.unit {
                MouseScrollUnit::Line => ev.y,
                MouseScrollUnit::Pixel => ev.y * CAMERA_SCROLL_PIXEL_FACTOR,
            };
        }
    }

    let Ok((mut cam, mut transform)) = query.single_mut() else { return };

    cam.yaw -= orbit_delta.x * CAMERA_ORBIT_SENSITIVITY;
    cam.pitch = (cam.pitch - orbit_delta.y * CAMERA_ORBIT_SENSITIVITY)
        .clamp(CAMERA_PITCH_MIN, CAMERA_PITCH_MAX);
    cam.radius = (cam.radius - zoom * CAMERA_ZOOM_SPEED)
        .clamp(CAMERA_RADIUS_MIN, CAMERA_RADIUS_MAX);

    if pan_delta != Vec2::ZERO {
        let scale = cam.radius * CAMERA_PAN_SENSITIVITY;
        cam.focus -= transform.right()   * pan_delta.x * scale;
        cam.focus += transform.up()      * pan_delta.y * scale;
    }

    let rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
    transform.translation = cam.focus + rotation * Vec3::new(0.0, 0.0, cam.radius);
    transform.look_at(cam.focus, Vec3::Y);
}
