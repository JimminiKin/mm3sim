use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;

#[derive(Component)]
pub struct ChuteSegment;

/// Profile point: (z, y) position and (tz, ty) forward tangent in param space.
/// Forward = direction marble travels (toward snare).
type ProfilePt = (f32, f32, f32, f32);

/// Builds the ordered list of profile points for the 3-part chute.
/// slope_start → arc_start → arc points → exit_start → exit_end
fn build_profile(params: &ChuteParams) -> Vec<ProfilePt> {
    let geo = params.geometry();
    let [slope_start_z, slope_start_y] = geo.slope_start;
    let [arc_start_z, arc_start_y] = geo.arc_start;
    let [center_z, center_y] = geo.center;
    let [exit_z, exit_y] = params.exit_pos;
    let [slope_tz, slope_ty] = geo.slope_tangent;
    let [exit_tz, exit_ty] = geo.exit_tangent;

    let sa = params.slope_angle.to_radians();
    let ea = params.exit_angle.to_radians();
    let arc_angle = (sa - ea).abs();
    let arc_len = params.curve_radius * arc_angle;
    // At least 4 arc segments, ~200 per metre of arc length, capped at 64.
    let n_arc = 4.max((arc_len * 200.0) as usize).min(64);

    let mut pts = Vec::with_capacity(n_arc + 3);

    // Straight slope (2 points)
    pts.push((slope_start_z, slope_start_y, slope_tz, slope_ty));
    pts.push((arc_start_z, arc_start_y, slope_tz, slope_ty));

    // Circular arc (n_arc additional points, ending at exit_start)
    for i in 1..=n_arc {
        let t = i as f32 / n_arc as f32;
        let theta = geo.theta_start + t * geo.arc_sweep;
        let z = center_z + params.curve_radius * theta.cos();
        let y = center_y + params.curve_radius * theta.sin();
        // Forward tangent on CW arc: (sin θ, -cos θ)
        let tz = theta.sin();
        let ty = -theta.cos();
        pts.push((z, y, tz, ty));
    }

    // Straight exit (1 more point for exit_end)
    pts.push((exit_z, exit_y, exit_tz, exit_ty));

    pts
}

/// Spawns one chute instance.
///
/// Geometry is built in snare-local space (profile in the Y-Z plane at X = 0,
/// Y origin = snare top face).  The entity `Transform` then:
/// - rotates the chute by `angle_rad` around the Y-axis (through the snare centre),
/// - translates by `snare_offset` to follow the snare assembly.
///
/// Because the body is `RigidBody::Static`, Avian3D applies the entity transform
/// to the trimesh collider automatically.
pub fn spawn_chute(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &ChuteParams,
    snare_offset: Vec3,
    angle_rad: f32,
) {
    let profile = build_profile(params);
    let (coll_verts, coll_idx) = build_trimesh_collider(&profile);

    commands.spawn((
        Mesh3d(meshes.add(build_smooth_mesh(&profile))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.45, 0.30),
            metallic: 0.0,
            perceptual_roughness: 0.65,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: snare_offset,
            rotation: Quat::from_rotation_y(angle_rad),
            scale: Vec3::ONE,
        },
        RigidBody::Static,
        Collider::trimesh_with_config(coll_verts, coll_idx, TrimeshFlags::FIX_INTERNAL_EDGES),
        Restitution::new(params.restitution).with_combine_rule(CoefficientCombine::Min),
        Friction::new(params.friction),
        ChuteSegment,
    ));
}

/// normal_3d for a point with forward tangent (tz, ty) in (Z, Y) param space.
/// Derived from CW rotation: normal = (0, -tz, ty) in world (X, Y, Z).
#[inline]
fn surface_normal(tz: f32, ty: f32) -> Vec3 {
    Vec3::new(0.0, -tz, ty).normalize_or_zero()
}

/// Build trimesh collider vertices in snare-local space (no snare_offset, no rotation).
fn build_trimesh_collider(profile: &[ProfilePt]) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let n = profile.len() - 1;
    let w = CHUTE_WIDTH * 0.5;
    let h = CHUTE_THICKNESS * 0.5;

    let mut verts: Vec<Vec3> = Vec::with_capacity((n + 1) * 4);
    for &(z, y, tz, ty) in profile {
        let sn = surface_normal(tz, ty);
        let centre = Vec3::new(CHUTE_END_X, y + CHUTE_ORIGIN_Y, z + CHUTE_ORIGIN_Z);
        let top = centre + sn * h;
        let bot = centre - sn * h;
        verts.push(Vec3::new(top.x - w, top.y, top.z));
        verts.push(Vec3::new(top.x + w, top.y, top.z));
        verts.push(Vec3::new(bot.x - w, bot.y, bot.z));
        verts.push(Vec3::new(bot.x + w, bot.y, bot.z));
    }

    let mut idx: Vec<[u32; 3]> = Vec::with_capacity(n * 8);
    for i in 0..n as u32 {
        let a = i * 4;
        let b = (i + 1) * 4;
        idx.push([a,   b+1, b  ]); idx.push([a,   a+1, b+1]);
        idx.push([a+3, b+3, b+2]); idx.push([a+3, b+2, a+2]);
        idx.push([a,   a+2, b+2]); idx.push([a,   b+2, b  ]);
        idx.push([a+1, b+1, b+3]); idx.push([a+1, b+3, a+3]);
    }

    (verts, idx)
}

/// Build visual mesh in snare-local space (no snare_offset, no rotation).
fn build_smooth_mesh(profile: &[ProfilePt]) -> Mesh {
    let n = profile.len() - 1;
    let w = CHUTE_WIDTH * 0.5;
    let h = CHUTE_THICKNESS * 0.5;

    const STRIDE: usize = 8;
    let cap = (n + 1) * STRIDE;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(cap);
    let mut normals:   Vec<[f32; 3]> = Vec::with_capacity(cap);
    let mut uvs:       Vec<[f32; 2]> = Vec::with_capacity(cap);

    let total_segs = n as f32;
    for (seg_i, &(z, y, tz, ty)) in profile.iter().enumerate() {
        let t = seg_i as f32 / total_segs;
        let sn = surface_normal(tz, ty);
        let bn = -sn;
        let centre = Vec3::new(CHUTE_END_X, y + CHUTE_ORIGIN_Y, z + CHUTE_ORIGIN_Z);
        let top = centre + sn * h;
        let bot = centre + bn * h;

        macro_rules! push {
            ($p:expr, $n:expr, $uv:expr) => {
                positions.push(($p).to_array());
                normals.push(($n).to_array());
                uvs.push($uv);
            };
        }

        push!(Vec3::new(top.x-w, top.y, top.z), sn,          [t, 0.0]);
        push!(Vec3::new(top.x+w, top.y, top.z), sn,          [t, 1.0]);
        push!(Vec3::new(bot.x-w, bot.y, bot.z), bn,          [t, 0.0]);
        push!(Vec3::new(bot.x+w, bot.y, bot.z), bn,          [t, 1.0]);
        push!(Vec3::new(top.x-w, top.y, top.z), Vec3::NEG_X, [t, 1.0]);
        push!(Vec3::new(bot.x-w, bot.y, bot.z), Vec3::NEG_X, [t, 0.0]);
        push!(Vec3::new(top.x+w, top.y, top.z), Vec3::X,     [t, 1.0]);
        push!(Vec3::new(bot.x+w, bot.y, bot.z), Vec3::X,     [t, 0.0]);
    }

    let mut indices: Vec<u32> = Vec::with_capacity(n * 4 * 6);
    for i in 0..n {
        let a = (i * STRIDE) as u32;
        let b = ((i + 1) * STRIDE) as u32;
        quad(&mut indices, a,   a+1, b,   b+1);
        quad(&mut indices, a+3, a+2, b+3, b+2);
        quad(&mut indices, b+4, b+5, a+4, a+5);
        quad(&mut indices, a+6, a+7, b+6, b+7);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[inline]
fn quad(idx: &mut Vec<u32>, a: u32, b: u32, c: u32, d: u32) {
    idx.extend_from_slice(&[a, b, d, a, d, c]);
}
