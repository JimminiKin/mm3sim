use avian3d::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

/// All geometry and physics needed to build one pivot-mounted arm assembly.
///
/// The pivot anchor is placed at `pivot_world_pos`.  The arm is spawned at its
/// rest angle (`−rest_deg` about the X axis) so the instrument end hangs lower.
///
/// # Conventions
/// * Arm local frame: +Z toward CW end, −Z toward instrument end.
/// * `pivot_local_z > 0`: pivot is between arm centre and the CW end.
/// * `rest_deg > 0`: instrument-side is lower at rest.
pub struct PivotArmSpec {
    /// Static world position of the revolute-joint pivot anchor.
    pub pivot_world_pos: Vec3,
    /// Half-length of the arm tube.
    /// Instrument attaches at arm local z = `−arm_half_len`; CW at `+arm_half_len`.
    pub arm_half_len: f32,
    /// Offset from arm centre to pivot along local Z (positive = toward CW end).
    pub pivot_local_z: f32,
    /// Rest angle in degrees (positive = instrument-side down).
    pub rest_deg: f32,
    /// Maximum additional downward tilt past rest.
    pub max_tilt_deg: f32,
    // ── Arm ──────────────────────────────────────────────────────────────────
    pub arm_mass: f32,
    pub arm_tube_radius: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    // ── Counterweight ────────────────────────────────────────────────────────
    pub cw_mass: f32,
    pub cw_radius: f32,
    pub cw_half_height: f32,
    // ── Pivot stand ──────────────────────────────────────────────────────────
    /// Height of the decorative stand cylinder below the pivot.
    /// Use `0.0` for instruments that have no stand (e.g. vibraphone bars).
    pub stand_half_height: f32,
}

/// Spawns a pivot-mounted arm assembly: static anchor, optional stand, arm tube,
/// counterweight, and revolute joint.
///
/// Returns the arm `Entity`.  The caller should:
/// 1. Insert instrument-specific marker components on the arm entity.
/// 2. Spawn the instrument surface as a child of the arm at local z = `−arm_half_len`.
///
/// The `marker` component (must be `Clone`) is attached to the anchor, stand,
/// arm, and joint so that a rebuild system can despawn the whole assembly by
/// querying `With<marker>`.  Arm children (tube, CW) are automatically removed
/// when the arm is despawned (Bevy 0.15+ hierarchical despawn).
pub fn spawn_pivot_arm<M: Component + Clone>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    spec: &PivotArmSpec,
    frame_material: Handle<StandardMaterial>,
    marker: M,
) -> Entity {
    let arm_rad = -spec.rest_deg.to_radians(); // negative → instrument end down
    let sin_r = arm_rad.sin();
    let cos_r = arm_rad.cos();

    // Arm centre world position at rest angle.
    // arm_centre = pivot − R_x(arm_rad) · (0, 0, pivot_local_z)
    // R_x(θ) · (0, 0, z) = (0, −z·sin θ, z·cos θ)
    let arm_world = Vec3::new(
        spec.pivot_world_pos.x,
        spec.pivot_world_pos.y + spec.pivot_local_z * sin_r, // sin_r < 0 → arm centre below pivot
        spec.pivot_world_pos.z - spec.pivot_local_z * cos_r,
    );

    // ── Static anchor at pivot ────────────────────────────────────────────────
    let anchor = commands
        .spawn((
            Transform::from_translation(spec.pivot_world_pos),
            RigidBody::Static,
            marker.clone(),
        ))
        .id();

    // ── Decorative stand below pivot (omitted when stand_half_height == 0) ───
    if spec.stand_half_height > 0.0 {
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cylinder {
                radius: spec.arm_tube_radius,
                half_height: spec.stand_half_height,
            }))),
            MeshMaterial3d(frame_material.clone()),
            Transform::from_xyz(
                spec.pivot_world_pos.x,
                spec.pivot_world_pos.y - spec.stand_half_height,
                spec.pivot_world_pos.z,
            ),
            marker.clone(),
        ));
    }

    // ── Dynamic arm ───────────────────────────────────────────────────────────
    let arm = commands
        .spawn((
            Transform::from_translation(arm_world).with_rotation(Quat::from_rotation_x(arm_rad)),
            Visibility::default(),
            RigidBody::Dynamic,
            LinearDamping(spec.linear_damping),
            AngularDamping(spec.angular_damping),
            marker.clone(),
        ))
        .with_children(|p| {
            // Arm tube (oriented along local Z via 90° rotation about X)
            p.spawn((
                Mesh3d(meshes.add(Mesh::from(Cylinder {
                    radius: spec.arm_tube_radius,
                    half_height: spec.arm_half_len,
                }))),
                MeshMaterial3d(frame_material.clone()),
                Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                Collider::cylinder(spec.arm_tube_radius, spec.arm_half_len * 2.0),
                Mass(spec.arm_mass),
            ));

            // Counterweight at the CW end (+Z)
            p.spawn((
                Mesh3d(meshes.add(Mesh::from(Cylinder {
                    radius: spec.cw_radius,
                    half_height: spec.cw_half_height,
                }))),
                MeshMaterial3d(frame_material.clone()),
                Transform::from_xyz(0.0, 0.0, spec.arm_half_len),
                Collider::cylinder(spec.cw_radius, spec.cw_half_height * 2.0),
                Mass(spec.cw_mass),
            ));
        })
        .id();

    // ── Revolute joint ────────────────────────────────────────────────────────
    commands.spawn((
        RevoluteJoint::new(anchor, arm)
            .with_hinge_axis(Vec3::X)
            .with_local_anchor1(Vec3::ZERO)
            .with_local_anchor2(Vec3::new(0.0, 0.0, spec.pivot_local_z))
            .with_angle_limits(
                -(spec.rest_deg + spec.max_tilt_deg).to_radians(),
                -spec.rest_deg.to_radians(),
            ),
        marker,
    ));

    arm
}
