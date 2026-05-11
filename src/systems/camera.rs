use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

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
            radius: 16.12,
            yaw: 0.0,
            pitch: -0.52,
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
            MouseScrollUnit::Pixel => ev.y * 0.1,
        };
    }

    let Ok((mut cam, mut transform)) = query.get_single_mut() else { return };

    cam.yaw -= delta.x * 0.005;
    cam.pitch = (cam.pitch - delta.y * 0.005).clamp(-1.5, 0.4);
    cam.radius = (cam.radius - zoom * 0.8).clamp(3.0, 60.0);

    let rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
    transform.translation = cam.focus + rotation * Vec3::new(0.0, 0.0, cam.radius);
    transform.look_at(cam.focus, Vec3::Y);
}
