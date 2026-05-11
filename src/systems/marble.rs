use bevy::prelude::*;
use bevy::math::primitives::Sphere;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

use crate::resources::constants::*;

pub fn spawn_marble_on_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = camera_query.single();
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let plane_y = 1.5;
    if ray.direction.y.abs() < 0.001 {
        return;
    }

    let t = (plane_y - ray.origin.y) / ray.direction.y;
    let target = ray.origin + ray.direction * t;
    let spawn_position = Vec3::new(target.x, SPAWN_HEIGHT, target.z);

    spawn_marble(&mut commands, &mut meshes, &mut materials, spawn_position);
}

pub fn spawn_marble(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere { radius: MARBLE_RADIUS })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.95, 0.35, 0.15),
                metallic: 0.8,
                perceptual_roughness: 0.2,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(MARBLE_RADIUS),
        Restitution::coefficient(0.75),
        Friction::coefficient(0.6),
        GravityScale(1.0),
        Velocity::default(),
    ));
}