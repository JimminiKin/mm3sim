//! Visual chute handles (endpoint spheres) and gizmo overlay.
//!
//! Two sphere markers show the exit end (red, handle 0) and slope start (green, handle 1).
//! Visibility is toggled by `ChuteParams::handles_visible`.

use bevy::math::primitives::Sphere;
use bevy::prelude::*;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::{CHUTE_END_X, CHUTE_ORIGIN_Y, CHUTE_ORIGIN_Z};
use crate::resources::snare_params::SnareParams;

const HANDLE_RADIUS: f32 = 0.012;

// Handle 0 = exit end (red), handle 1 = slope start (green)
const COLORS: [Color; 2] = [
    Color::srgba(0.90, 0.10, 0.10, 0.35), // exit end – red
    Color::srgba(0.20, 0.90, 0.20, 0.35), // slope start – green
];

#[derive(Component)]
pub struct ChuteHandle(pub usize); // 0 = exit end, 1 = slope start

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
