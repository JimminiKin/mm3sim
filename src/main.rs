use bevy::prelude::*;
use bevy::math::primitives::{Cuboid, Sphere};
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

const PLATE_SIZE: Vec3 = Vec3::new(8.0, 0.2, 4.0);
const MARBLE_RADIUS: f32 = 0.25;
const SPAWN_HEIGHT: f32 = 8.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.08)))
        .add_systems(Startup, setup_system)
        .add_systems(Update, spawn_marble_on_click_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 8.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 25_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.7, 0.0)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        brightness: 0.35,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(
                PLATE_SIZE.x,
                PLATE_SIZE.y,
                PLATE_SIZE.z,
            ))),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.15, 0.45, 0.80),
                metallic: 0.2,
                perceptual_roughness: 0.5,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(PLATE_SIZE.x / 2.0, PLATE_SIZE.y / 2.0, PLATE_SIZE.z / 2.0),
    ));
}

fn spawn_marble_on_click_system(
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

fn spawn_marble(
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
