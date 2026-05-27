use bevy::{
    math::primitives::Cuboid,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use bevy::camera::Viewport;
use bevy::camera::visibility::RenderLayers;

use crate::systems::camera::OrbitCamera;

#[derive(Component)]
pub struct AxesCamera;

#[derive(Component)]
pub struct AxisLabel(usize); // 0=X 1=Y 2=Z

const AXIS_LEN: f32 = 0.75;
const SHAFT_THICK: f32 = 0.055;
const HEAD_LEN: f32 = 0.18;
const HEAD_WIDE: f32 = 0.12;
const CAM_DIST: f32 = 3.5;
pub const WIDGET_PX: u32 = 120;

pub fn setup_axes_hud(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single().expect("primary window");
    let pw = window.physical_width();
    let ph = window.physical_height();

    // ── Secondary camera — bottom-right corner ────────────────────────────────
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            viewport: Some(Viewport {
                physical_position: UVec2::new(
                    pw.saturating_sub(WIDGET_PX + 10),
                    ph.saturating_sub(WIDGET_PX + 10),
                ),
                physical_size: UVec2::new(WIDGET_PX, WIDGET_PX),
                ..default()
            }),
            clear_color: ClearColorConfig::Custom(Color::srgba(0.06, 0.06, 0.10, 1.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, CAM_DIST).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
        AxesCamera,
    ));

    // ── Axis arrows (shaft + arrowhead per axis) ──────────────────────────────
    let axis_defs = [
        (Vec3::X, Color::srgb(0.90, 0.18, 0.18)),
        (Vec3::Y, Color::srgb(0.18, 0.85, 0.18)),
        (Vec3::Z, Color::srgb(0.20, 0.40, 0.95)),
    ];

    for (dir, color) in axis_defs {
        let mat = materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        });
        let perp = Vec3::ONE - dir.abs();

        let shaft_sz = dir * AXIS_LEN + perp * SHAFT_THICK;
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(shaft_sz.x, shaft_sz.y, shaft_sz.z)))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(dir * (AXIS_LEN * 0.5)),
            RenderLayers::layer(1),
        ));

        let head_sz = dir * HEAD_LEN + perp * HEAD_WIDE;
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(head_sz.x, head_sz.y, head_sz.z)))),
            MeshMaterial3d(mat),
            Transform::from_translation(dir * (AXIS_LEN + HEAD_LEN * 0.5)),
            RenderLayers::layer(1),
        ));
    }

    // ── Letter labels (2D UI, positioned each frame via projection) ───────────
    let label_colors = [
        Color::srgb(1.0, 0.45, 0.45),
        Color::srgb(0.45, 1.0, 0.45),
        Color::srgb(0.45, 0.65, 1.0),
    ];
    for (i, (letter, color)) in ["X", "Y", "Z"].iter().zip(label_colors.iter()).enumerate() {
        commands.spawn((
            Text::new(*letter),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(*color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-100.0),
                left: Val::Px(-100.0),
                ..default()
            },
            AxisLabel(i),
        ));
    }
}

pub fn update_axes_hud(
    main_cam: Query<&GlobalTransform, With<OrbitCamera>>,
    mut axes_transform_q: Query<&mut Transform, With<AxesCamera>>,
    axes_cam_q: Query<(&Camera, &GlobalTransform), With<AxesCamera>>,
    mut labels: Query<(&mut Node, &AxisLabel)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(main_gt) = main_cam.single() else { return };
    let Ok(mut axes_tf) = axes_transform_q.single_mut() else { return };
    let Ok((axes_camera, axes_gt)) = axes_cam_q.single() else { return };

    // Mirror main camera orientation: secondary camera orbits the origin identically
    let (_, main_rot, _) = main_gt.to_scale_rotation_translation();
    axes_tf.rotation = main_rot;
    axes_tf.translation = main_rot * (Vec3::Z * CAM_DIST);

    // Compute viewport logical offset for label placement
    let scale = windows.single().map(|w| w.scale_factor()).unwrap_or(1.0) as f32;
    let vp_offset = axes_camera
        .viewport
        .as_ref()
        .map(|vp| Vec2::new(
            vp.physical_position.x as f32 / scale,
            vp.physical_position.y as f32 / scale,
        ))
        .unwrap_or(Vec2::ZERO);

    // Project just past each arrowhead tip
    let tip_positions = [
        Vec3::X * (AXIS_LEN + HEAD_LEN + 0.08),
        Vec3::Y * (AXIS_LEN + HEAD_LEN + 0.08),
        Vec3::Z * (AXIS_LEN + HEAD_LEN + 0.08),
    ];

    for (mut node, label) in &mut labels {
        match axes_camera.world_to_viewport(axes_gt, tip_positions[label.0]) {
            Ok(pos) => {
                node.left = Val::Px(vp_offset.x + pos.x - 5.0);
                node.top = Val::Px(vp_offset.y + pos.y - 7.0);
            }
            Err(_) => {
                node.left = Val::Px(-100.0);
                node.top = Val::Px(-100.0);
            }
        }
    }
}

/// Keeps the axes widget pinned to the bottom-right corner after a window resize.
pub fn resize_axes_viewport(
    mut events: MessageReader<WindowResized>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut axes_cam: Query<&mut Camera, With<AxesCamera>>,
) {
    for _ in events.read() {
        let Ok(window) = windows.single() else { continue };
        let pw = window.physical_width();
        let ph = window.physical_height();
        let Ok(mut cam) = axes_cam.single_mut() else { continue };
        if let Some(vp) = &mut cam.viewport {
            vp.physical_position = UVec2::new(
                pw.saturating_sub(WIDGET_PX + 10),
                ph.saturating_sub(WIDGET_PX + 10),
            );
        }
    }
}
