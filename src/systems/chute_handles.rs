use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::CHUTE_END_X;
use crate::systems::camera::OrbitCamera;

const HANDLE_RADIUS: f32 = 0.12;
const PICK_RADIUS: f32 = 0.25;

const COLORS: [Color; 4] = [
    Color::rgb(0.20, 0.90, 0.20), // P0  – green
    Color::rgb(0.95, 0.80, 0.10), // CP1 – yellow
    Color::rgb(0.95, 0.45, 0.10), // CP2 – orange
    Color::rgb(0.90, 0.10, 0.10), // P3  – red
];

#[derive(Component)]
pub struct ChuteHandle(pub usize); // 0=P0, 1=CP1, 2=CP2, 3=P3

#[derive(Resource, Default)]
pub struct HandleDrag {
    pub active: Option<usize>,
}

pub fn setup_chute_handles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    params: Res<ChuteParams>,
) {
    let pts = [params.p0, params.cp1, params.cp2, params.p3];
    for (i, (pt, &color)) in pts.iter().zip(COLORS.iter()).enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Sphere { radius: HANDLE_RADIUS })),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_xyz(CHUTE_END_X, pt[1], pt[0]),
                ..default()
            },
            ChuteHandle(i),
        ));
    }
}

/// Keep handle sphere positions in sync with ChuteParams every frame.
pub fn sync_handle_transforms(
    params: Res<ChuteParams>,
    mut handles: Query<(&ChuteHandle, &mut Transform)>,
) {
    for (handle, mut tf) in &mut handles {
        let pt = match handle.0 {
            0 => params.p0,
            1 => params.cp1,
            2 => params.cp2,
            _ => params.p3,
        };
        tf.translation = Vec3::new(CHUTE_END_X, pt[1], pt[0]);
    }
}

pub fn chute_handle_drag_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    handles: Query<(&ChuteHandle, &Transform)>,
    mut params: ResMut<ChuteParams>,
    mut drag: ResMut<HandleDrag>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_gt)) = camera_q.get_single() else { return };
    let Some(ray) = camera.viewport_to_world(cam_gt, cursor) else { return };

    let origin = ray.origin;
    let dir = Vec3::from(ray.direction);

    // ── Pick on press ────────────────────────────────────────────────────────
    if buttons.just_pressed(MouseButton::Left) {
        let mut best: Option<(usize, f32)> = None;
        for (handle, tf) in &handles {
            if let Some(t) = ray_sphere(origin, dir, tf.translation, PICK_RADIUS) {
                if best.map_or(true, |(_, bt)| t < bt) {
                    best = Some((handle.0, t));
                }
            }
        }
        drag.active = best.map(|(idx, _)| idx);
    }

    // ── Release: trigger segment rebuild ────────────────────────────────────
    if buttons.just_released(MouseButton::Left) {
        if drag.active.is_some() {
            params.dirty = true;
        }
        drag.active = None;
    }

    // ── Drag: update params (handles sync via sync_handle_transforms) ────────
    if let Some(idx) = drag.active {
        if let Some(hit) = ray_x_plane(origin, dir, CHUTE_END_X) {
            let pt = match idx {
                0 => &mut params.p0,
                1 => &mut params.cp1,
                2 => &mut params.cp2,
                _ => &mut params.p3,
            };
            pt[0] = hit.z;
            pt[1] = hit.y;
            // dirty NOT set here — segments stay until mouse release
        }
    }
}

/// Draw tangent arms and Bézier curve preview using gizmos.
pub fn draw_chute_gizmos(params: Res<ChuteParams>, mut gizmos: Gizmos) {
    let x = CHUTE_END_X;
    let p0  = Vec3::new(x, params.p0[1],  params.p0[0]);
    let cp1 = Vec3::new(x, params.cp1[1], params.cp1[0]);
    let cp2 = Vec3::new(x, params.cp2[1], params.cp2[0]);
    let p3  = Vec3::new(x, params.p3[1],  params.p3[0]);

    // Tangent arms
    gizmos.line(p0, cp1, Color::rgb(0.95, 0.85, 0.20));
    gizmos.line(p3, cp2, Color::rgb(0.95, 0.85, 0.20));

    // Curve preview
    let n = 64usize;
    let mut prev = p0;
    for i in 1..=n {
        let t = i as f32 / n as f32;
        let next = bezier(p0, cp1, cp2, p3, t);
        gizmos.line(prev, next, Color::rgb(0.35, 0.75, 1.0));
        prev = next;
    }
}

// ── Math helpers ─────────────────────────────────────────────────────────────

fn bezier(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    u*u*u*p0 + 3.0*u*u*t*p1 + 3.0*u*t*t*p2 + t*t*t*p3
}

/// Returns distance along ray to sphere hit, or None.
fn ray_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let b = oc.dot(dir);
    let c = oc.length_squared() - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 { return None; }
    let t = -b - disc.sqrt();
    if t < 0.0 { return None; }
    Some(t)
}

/// Returns intersection point of ray with the plane x = plane_x.
fn ray_x_plane(origin: Vec3, dir: Vec3, plane_x: f32) -> Option<Vec3> {
    if dir.x.abs() < 1e-6 { return None; }
    let t = (plane_x - origin.x) / dir.x;
    if t < 0.0 { return None; }
    Some(origin + dir * t)
}
