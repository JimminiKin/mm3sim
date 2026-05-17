use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_rapier3d::prelude::*;

use crate::resources::chute_params::ChuteParams;
use crate::resources::constants::*;

#[derive(Component)]
pub struct ChuteSegment;

// ── Bézier helpers ────────────────────────────────────────────────────────────

fn bz(pts: [[f32; 2]; 4], t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let [p0, p1, p2, p3] = pts;
    (
        u*u*u*p0[0] + 3.0*u*u*t*p1[0] + 3.0*u*t*t*p2[0] + t*t*t*p3[0],
        u*u*u*p0[1] + 3.0*u*u*t*p1[1] + 3.0*u*t*t*p2[1] + t*t*t*p3[1],
    )
}

/// cross(X, tangent_3d) = (0, −dz, dy), normalised — points toward the sliding surface.
fn surface_normal(pts: [[f32; 2]; 4], t: f32) -> Vec3 {
    let u = 1.0 - t;
    let [p0, p1, p2, p3] = pts;
    let dz = 3.0 * (u*u*(p1[0]-p0[0]) + 2.0*u*t*(p2[0]-p1[0]) + t*t*(p3[0]-p2[0]));
    let dy = 3.0 * (u*u*(p1[1]-p0[1]) + 2.0*u*t*(p2[1]-p1[1]) + t*t*(p3[1]-p2[1]));
    Vec3::new(0.0, -dz, dy).normalize_or_zero()
}

// ── Spawn ─────────────────────────────────────────────────────────────────────

pub fn spawn_cycloid_chute(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &ChuteParams,
) {
    let pts = params.effective_pts();

    let (coll_verts, coll_idx) = build_trimesh_collider(pts);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(build_smooth_mesh(pts)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.45, 0.30),
                metallic: 0.0,
                perceptual_roughness: 0.65,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            ..default()
        },
        RigidBody::Fixed,
        Collider::trimesh(coll_verts, coll_idx),
        Restitution {
            coefficient: CHUTE_RESTITUTION,
            combine_rule: CoefficientCombineRule::Min,
        },
        Friction::coefficient(CHUTE_FRICTION),
        ChuteSegment,
    ));
}

// ── Trimesh collider ──────────────────────────────────────────────────────────

/// Builds a closed-shell trimesh that exactly matches the curve.
/// 4 unique positions per cross-section (TL, TR, BL, BR).
fn build_trimesh_collider(pts: [[f32; 2]; 4]) -> (Vec<Vec3>, Vec<[u32; 3]>) {
    let n = CHUTE_SEGMENTS; // physics resolution
    let w = CHUTE_WIDTH * 0.5;
    let h = CHUTE_THICKNESS * 0.5;
    let x = CHUTE_END_X;

    let mut verts: Vec<Vec3> = Vec::with_capacity((n + 1) * 4);

    for i in 0..=(n) {
        let t = i as f32 / n as f32;
        let (z, y) = bz(pts, t);
        let sn = surface_normal(pts, t);

        let centre = Vec3::new(x, y + CHUTE_ORIGIN_Y, z + CHUTE_ORIGIN_Z);
        let top = centre + sn * h;
        let bot = centre - sn * h;

        verts.push(Vec3::new(top.x - w, top.y, top.z)); // i*4+0 TL
        verts.push(Vec3::new(top.x + w, top.y, top.z)); // i*4+1 TR
        verts.push(Vec3::new(bot.x - w, bot.y, bot.z)); // i*4+2 BL
        verts.push(Vec3::new(bot.x + w, bot.y, bot.z)); // i*4+3 BR
    }

    let mut idx: Vec<[u32; 3]> = Vec::with_capacity(n * 8);

    for i in 0..n as u32 {
        let a = i * 4;
        let b = (i + 1) * 4;

        // top
        idx.push([a,   b,   b+1]); idx.push([a,   b+1, a+1]);
        // bottom
        idx.push([a+3, b+3, b+2]); idx.push([a+3, b+2, a+2]);
        // left
        idx.push([a,   a+2, b+2]); idx.push([a,   b+2, b  ]);
        // right
        idx.push([a+1, b+1, b+3]); idx.push([a+1, b+3, a+3]);
    }

    (verts, idx)
}

// ── Smooth visual mesh ────────────────────────────────────────────────────────

/// Ribbon mesh with smooth normals along the curve — 4× more samples than the collider.
fn build_smooth_mesh(pts: [[f32; 2]; 4]) -> Mesh {
    let n = CHUTE_SEGMENTS * 4;
    let w = CHUTE_WIDTH * 0.5;
    let h = CHUTE_THICKNESS * 0.5;
    let x = CHUTE_END_X;

    // 8 verts/section for sharp side edges with correct per-face normals:
    //  0,1 – top   (L, R)  normal = +sn
    //  2,3 – bot   (L, R)  normal = −sn
    //  4,5 – left  (T, B)  normal = −X
    //  6,7 – right (T, B)  normal = +X
    const STRIDE: usize = 8;

    let cap = (n + 1) * STRIDE;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(cap);
    let mut normals:   Vec<[f32; 3]> = Vec::with_capacity(cap);
    let mut uvs:       Vec<[f32; 2]> = Vec::with_capacity(cap);

    for i in 0..=(n) {
        let t = i as f32 / n as f32;
        let (z, y) = bz(pts, t);
        let sn = surface_normal(pts, t);
        let bn = -sn;

        let centre = Vec3::new(x, y + CHUTE_ORIGIN_Y, z + CHUTE_ORIGIN_Z);
        let top = centre + sn * h;
        let bot = centre + bn * h;
        let u = t;

        let push = |positions: &mut Vec<[f32; 3]>,
                    normals: &mut Vec<[f32; 3]>,
                    uvs: &mut Vec<[f32; 2]>,
                    p: Vec3, n: Vec3, uv: [f32; 2]| {
            positions.push(p.to_array());
            normals.push(n.to_array());
            uvs.push(uv);
        };

        push(&mut positions, &mut normals, &mut uvs, Vec3::new(top.x-w,top.y,top.z), sn, [u,0.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(top.x+w,top.y,top.z), sn, [u,1.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(bot.x-w,bot.y,bot.z), bn, [u,0.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(bot.x+w,bot.y,bot.z), bn, [u,1.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(top.x-w,top.y,top.z), Vec3::NEG_X, [u,1.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(bot.x-w,bot.y,bot.z), Vec3::NEG_X, [u,0.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(top.x+w,top.y,top.z), Vec3::X,     [u,1.0]);
        push(&mut positions, &mut normals, &mut uvs, Vec3::new(bot.x+w,bot.y,bot.z), Vec3::X,     [u,0.0]);
    }

    let mut indices: Vec<u32> = Vec::with_capacity(n * 4 * 6);
    for i in 0..n {
        let a = (i * STRIDE) as u32;
        let b = ((i + 1) * STRIDE) as u32;
        quad(&mut indices, a,   a+1, b,   b+1); // top
        quad(&mut indices, a+3, a+2, b+3, b+2); // bot
        quad(&mut indices, b+4, b+5, a+4, a+5); // left
        quad(&mut indices, a+6, a+7, b+6, b+7); // right
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
