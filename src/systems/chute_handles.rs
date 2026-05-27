use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::resources::chute_params::{ChuteParams, DragAxis};
use crate::resources::constants::{CHUTE_END_X, CHUTE_ORIGIN_Y, CHUTE_ORIGIN_Z};
use crate::resources::snare_params::SnareParams;
use crate::systems::camera::OrbitCamera;

const HANDLE_RADIUS: f32 = 0.012;
const PICK_RADIUS: f32 = 0.025;
const PICK_BODY_RADIUS: f32 = 0.035;

// Handle 0 = exit end (red), handle 1 = slope start (green)
const COLORS: [Color; 2] = [
    Color::srgba(0.90, 0.10, 0.10, 0.35), // exit end – red
    Color::srgba(0.20, 0.90, 0.20, 0.35), // slope start – green
];

#[derive(Component)]
pub struct ChuteHandle(pub usize); // 0 = exit end, 1 = slope start

pub enum DragState {
    Handle(usize),
    Body {
        anchor: [f32; 2],
        initial_exit_pos: [f32; 2],
    },
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
    let geo = params.geometry();
    let positions = [params.exit_pos, geo.slope_start];
    for (i, (pt, &color)) in positions.iter().zip(COLORS.iter()).enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Sphere { radius: HANDLE_RADIUS }))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(
                CHUTE_END_X,
                pt[1] + CHUTE_ORIGIN_Y,
                pt[0] + CHUTE_ORIGIN_Z,
            ),
            ChuteHandle(i),
        ));
    }
}

pub fn sync_handle_visibility(
    params: Res<ChuteParams>,
    mut handles: Query<&mut Visibility, With<ChuteHandle>>,
) {
    if !params.is_changed() {
        return;
    }
    let vis = if params.handles_visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in &mut handles {
        *v = vis;
    }
}

pub fn sync_handle_transforms(
    params: Res<ChuteParams>,
    snare_params: Res<SnareParams>,
    mut handles: Query<(&ChuteHandle, &mut Transform)>,
) {
    let sp = snare_params.pos;
    let geo = params.geometry();
    for (handle, mut tf) in &mut handles {
        let pt = match handle.0 {
            0 => params.exit_pos,
            _ => geo.slope_start,
        };
        tf.translation = Vec3::new(
            CHUTE_END_X + sp.x,
            pt[1] + CHUTE_ORIGIN_Y + sp.y,
            pt[0] + CHUTE_ORIGIN_Z + sp.z,
        );
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
    snare_params: Res<SnareParams>,
) {
    if contexts.ctx_mut().unwrap().wants_pointer_input() {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor) = window.cursor_position() else { return; };
    let Ok((camera, cam_gt)) = camera_q.single() else { return; };
    let Ok(ray) = camera.viewport_to_world(cam_gt, cursor) else { return; };

    let origin = ray.origin;
    let dir = Vec3::from(ray.direction);

    // ── Pick on press ────────────────────────────────────────────────────────
    if buttons.just_pressed(MouseButton::Left) {
        drag.active = None;

        // Test handle spheres (only when visible)
        if params.handles_visible {
            let mut best: Option<(usize, f32)> = None;
            for (handle, tf) in &handles {
                if let Some(t) = ray_sphere(origin, dir, tf.translation, PICK_RADIUS) {
                    if best.map_or(true, |(_, bt)| t < bt) {
                        best = Some((handle.0, t));
                    }
                }
            }
            if let Some((idx, _)) = best {
                drag.active = Some(DragState::Handle(idx));
            }
        }

        // Fall back: body drag if click is near the chute surface
        if drag.active.is_none() {
            let sp = snare_params.pos;
            if let Some(hit) = ray_x_plane(origin, dir, CHUTE_END_X + sp.x) {
                let hz = hit.z - CHUTE_ORIGIN_Z - sp.z;
                let hy = hit.y - CHUTE_ORIGIN_Y - sp.y;
                if dist_to_chute(&params, hz, hy) < PICK_BODY_RADIUS {
                    drag.active = Some(DragState::Body {
                        anchor: [hit.z, hit.y],
                        initial_exit_pos: params.exit_pos,
                    });
                }
            }
        }
    }

    // ── Release ──────────────────────────────────────────────────────────────
    if buttons.just_released(MouseButton::Left) {
        if drag.active.is_some() {
            params.dirty = true;
        }
        drag.active = None;
    }

    // ── Drag update ──────────────────────────────────────────────────────────
    let sp = snare_params.pos;
    if drag.active.is_some() {
        if let Some(hit) = ray_x_plane(origin, dir, CHUTE_END_X + sp.x) {
            let raw_hz = hit.z - CHUTE_ORIGIN_Z - sp.z;
            let raw_hy = hit.y - CHUTE_ORIGIN_Y - sp.y;

            match &drag.active {
                Some(DragState::Handle(0)) => {
                    // Exit handle: move exit_pos directly
                    match params.drag_axis {
                        DragAxis::Free       => { params.exit_pos = [raw_hz, raw_hy]; }
                        DragAxis::Vertical   => { params.exit_pos[1] = raw_hy; }
                        DragAxis::Horizontal => { params.exit_pos[0] = raw_hz; }
                    }
                }
                Some(DragState::Handle(_)) => {
                    // Slope handle: project onto slope direction, update slope_length only.
                    // The slope outward direction (from arc_start toward slope_start) is
                    // (cos(sa), sin(sa)) in (Z, Y) space.
                    let geo = params.geometry();
                    let [az, ay] = geo.arc_start;
                    let sa = params.slope_angle.to_radians();
                    let dir_z = sa.cos(); // outward direction (away from snare, upward)
                    let dir_y = sa.sin();

                    let (hz, hy) = match params.drag_axis {
                        DragAxis::Free       => (raw_hz, raw_hy),
                        DragAxis::Vertical   => (geo.slope_start[0], raw_hy),
                        DragAxis::Horizontal => (raw_hz, geo.slope_start[1]),
                    };

                    // Project displacement onto outward slope direction
                    let proj = (hz - az) * dir_z + (hy - ay) * dir_y;
                    params.slope_length = proj.max(0.01);
                }
                Some(DragState::Body { anchor, initial_exit_pos }) => {
                    let raw_dz = hit.z - anchor[0];
                    let raw_dy = hit.y - anchor[1];
                    let (dz, dy) = match params.drag_axis {
                        DragAxis::Free       => (raw_dz, raw_dy),
                        DragAxis::Vertical   => (0.0, raw_dy),
                        DragAxis::Horizontal => (raw_dz, 0.0),
                    };
                    params.exit_pos = [initial_exit_pos[0] + dz, initial_exit_pos[1] + dy];
                }
                None => {}
            }
        }
    }
}

/// Draw the 3-part chute profile as a gizmo overlay.
pub fn draw_chute_gizmos(
    params: Res<ChuteParams>,
    snare_params: Res<SnareParams>,
    mut gizmos: Gizmos,
) {
    let sp = snare_params.pos;
    let x = CHUTE_END_X + sp.x;
    let to_world = |z: f32, y: f32| Vec3::new(x, y + CHUTE_ORIGIN_Y + sp.y, z + CHUTE_ORIGIN_Z + sp.z);

    let geo = params.geometry();
    let color = Color::srgb(0.35, 0.75, 1.0);

    // Slope
    gizmos.line(
        to_world(geo.slope_start[0], geo.slope_start[1]),
        to_world(geo.arc_start[0], geo.arc_start[1]),
        color,
    );

    // Arc (32 segments)
    let n_giz = 32usize;
    let mut prev = to_world(geo.arc_start[0], geo.arc_start[1]);
    for i in 1..=n_giz {
        let t = i as f32 / n_giz as f32;
        let theta = geo.theta_start + t * geo.arc_sweep;
        let z = geo.center[0] + params.curve_radius * theta.cos();
        let y = geo.center[1] + params.curve_radius * theta.sin();
        let next = to_world(z, y);
        gizmos.line(prev, next, color);
        prev = next;
    }

    // Exit
    gizmos.line(prev, to_world(params.exit_pos[0], params.exit_pos[1]), color);
}

// ── Math helpers ─────────────────────────────────────────────────────────────

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

fn ray_x_plane(origin: Vec3, dir: Vec3, plane_x: f32) -> Option<Vec3> {
    if dir.x.abs() < 1e-6 { return None; }
    let t = (plane_x - origin.x) / dir.x;
    if t < 0.0 { return None; }
    Some(origin + dir * t)
}

/// Minimum distance from (z, y) in param space to the chute profile.
fn dist_to_chute(params: &ChuteParams, z: f32, y: f32) -> f32 {
    let geo = params.geometry();
    let n = 48u32;

    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            // Sample evenly across the 3 sections (slope / arc / exit)
            let (pz, py) = if t < 1.0 / 3.0 {
                let s = t * 3.0;
                let [sz, sy] = geo.slope_start;
                let [az, ay] = geo.arc_start;
                (sz + s * (az - sz), sy + s * (ay - sy))
            } else if t < 2.0 / 3.0 {
                let s = (t - 1.0 / 3.0) * 3.0;
                let theta = geo.theta_start + s * geo.arc_sweep;
                (
                    geo.center[0] + params.curve_radius * theta.cos(),
                    geo.center[1] + params.curve_radius * theta.sin(),
                )
            } else {
                let s = (t - 2.0 / 3.0) * 3.0;
                let [es_z, es_y] = geo.exit_start;
                let [ee_z, ee_y] = params.exit_pos;
                (es_z + s * (ee_z - es_z), es_y + s * (ee_y - es_y))
            };
            ((pz - z).powi(2) + (py - y).powi(2)).sqrt()
        })
        .fold(f32::MAX, f32::min)
}
