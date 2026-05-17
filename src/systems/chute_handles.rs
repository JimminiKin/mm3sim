use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::{CHUTE_END_X, CHUTE_ORIGIN_Y, CHUTE_ORIGIN_Z};
use crate::systems::camera::OrbitCamera;

const HANDLE_RADIUS: f32 = 0.012;
const PICK_RADIUS: f32 = 0.025;
const PICK_BODY_RADIUS: f32 = 0.035;

const COLORS: [Color; 4] = [
    Color::srgb(0.20, 0.90, 0.20), // P0  – green
    Color::srgb(0.95, 0.80, 0.10), // CP1 – yellow
    Color::srgb(0.95, 0.45, 0.10), // CP2 – orange
    Color::srgb(0.90, 0.10, 0.10), // P3  – red
];

#[derive(Component)]
pub struct ChuteHandle(pub usize); // 0=P0, 1=CP1, 2=CP2, 3=P3

pub enum DragState {
    Handle(usize),
    /// anchor and initial are stored as [z, y] matching ChuteParams point layout
    Body { anchor: [f32; 2], initial: [[f32; 2]; 4] },
}

#[derive(Resource, Default)]
pub struct HandleDrag {
    pub active: Option<DragState>,
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
                transform: Transform::from_xyz(CHUTE_END_X, pt[1] + CHUTE_ORIGIN_Y, pt[0] + CHUTE_ORIGIN_Z),
                ..default()
            },
            ChuteHandle(i),
        ));
    }
}

/// Show or hide handle spheres when the flag changes.
pub fn sync_handle_visibility(
    params: Res<ChuteParams>,
    mut handles: Query<&mut Visibility, With<ChuteHandle>>,
) {
    if !params.is_changed() {
        return;
    }
    let vis = if params.handles_visible { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in &mut handles {
        *v = vis;
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
        tf.translation = Vec3::new(CHUTE_END_X, pt[1] + CHUTE_ORIGIN_Y, pt[0] + CHUTE_ORIGIN_Z);
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
    if !params.handles_visible {
        drag.active = None;
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
        drag.active = if let Some((idx, _)) = best {
            Some(DragState::Handle(idx))
        } else if let Some(hit) = ray_x_plane(origin, dir, CHUTE_END_X) {
            // No handle hit — check if we clicked on the chute body.
            // Convert world hit to param space before testing.
            if dist_to_bezier(&params, hit.z - CHUTE_ORIGIN_Z, hit.y - CHUTE_ORIGIN_Y) < PICK_BODY_RADIUS {
                Some(DragState::Body {
                    anchor: [hit.z - CHUTE_ORIGIN_Z, hit.y - CHUTE_ORIGIN_Y],
                    initial: [params.p0, params.cp1, params.cp2, params.p3],
                })
            } else {
                None
            }
        } else {
            None
        };
    }

    // ── Release: trigger segment rebuild ────────────────────────────────────
    if buttons.just_released(MouseButton::Left) {
        if drag.active.is_some() {
            params.dirty = true;
        }
        drag.active = None;
    }

    // ── Drag: update params (handles sync via sync_handle_transforms) ────────
    if drag.active.is_some() {
        if let Some(hit) = ray_x_plane(origin, dir, CHUTE_END_X) {
            match &drag.active {
                Some(DragState::Handle(idx)) => {
                    let pt = match idx {
                        0 => &mut params.p0,
                        1 => &mut params.cp1,
                        2 => &mut params.cp2,
                        _ => &mut params.p3,
                    };
                    pt[0] = hit.z - CHUTE_ORIGIN_Z;
                    pt[1] = hit.y - CHUTE_ORIGIN_Y;
                }
                Some(DragState::Body { anchor, initial }) => {
                    let dz = hit.z - anchor[0];
                    let dy = hit.y - anchor[1];
                    params.p0  = [initial[0][0] + dz, initial[0][1] + dy];
                    params.cp1 = [initial[1][0] + dz, initial[1][1] + dy];
                    params.cp2 = [initial[2][0] + dz, initial[2][1] + dy];
                    params.p3  = [initial[3][0] + dz, initial[3][1] + dy];
                }
                None => {}
            }
        }
    }
}

/// Draw tangent arms and Bézier curve preview using gizmos.
pub fn draw_chute_gizmos(params: Res<ChuteParams>, mut gizmos: Gizmos) {
    let x = CHUTE_END_X;
    let to_world = |pt: [f32; 2]| Vec3::new(x, pt[1] + CHUTE_ORIGIN_Y, pt[0] + CHUTE_ORIGIN_Z);
    let pts = params.effective_pts();
    let p0  = to_world(pts[0]);
    let cp1 = to_world(pts[1]);
    let cp2 = to_world(pts[2]);
    let p3  = to_world(pts[3]);

    // Tangent arms — only meaningful in curve mode
    if !params.straight {
        gizmos.line(p0, to_world(params.cp1), Color::srgb(0.95, 0.85, 0.20));
        gizmos.line(p3, to_world(params.cp2), Color::srgb(0.95, 0.85, 0.20));
    }

    // Curve preview
    let n = 64usize;
    let mut prev = p0;
    for i in 1..=n {
        let t = i as f32 / n as f32;
        let next = bezier(p0, cp1, cp2, p3, t);
        gizmos.line(prev, next, Color::srgb(0.35, 0.75, 1.0));
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

/// Minimum distance from (z, y) to the Bézier curve defined by params.
fn dist_to_bezier(params: &ChuteParams, z: f32, y: f32) -> f32 {
    let pts = [params.p0, params.cp1, params.cp2, params.p3];
    let n = 32u32;
    (0..=n).map(|i| {
        let t = i as f32 / n as f32;
        let u = 1.0 - t;
        let [p0, p1, p2, p3] = pts;
        let bz = u*u*u*p0[0] + 3.0*u*u*t*p1[0] + 3.0*u*t*t*p2[0] + t*t*t*p3[0];
        let by = u*u*u*p0[1] + 3.0*u*u*t*p1[1] + 3.0*u*t*t*p2[1] + t*t*t*p3[1];
        ((bz - z).powi(2) + (by - y).powi(2)).sqrt()
    }).fold(f32::MAX, f32::min)
}
