use avian3d::prelude::*;
use bevy::math::primitives::{Cuboid, Cylinder};
use bevy::prelude::*;

use crate::components::instrument::Instrument;
use crate::resources::carousel_params::CarouselParams;
use crate::resources::constants::*;
use crate::resources::layers::GameLayer;

/// Tags every entity that belongs to the carousel assembly.
#[derive(Component)]
pub struct CarouselPart;

/// Tags the central X-axis cylinder (visual only).
#[derive(Component)]
pub struct CarouselBody;

/// Marks each of the 4 carousel instrument entities (0=crash, 1=cowbell, 2=tambourine, 3=woodblock).
#[derive(Component)]
pub struct CarouselSlot(pub u8);

/// Marks the visual arm cylinder for each slot — updated alongside its instrument.
#[derive(Component)]
pub struct CarouselArm(pub u8);

// Hit channels for the 4 carousel instruments — NOT in CHANNEL_DEFS and NOT spawn channels.
pub const CAROUSEL_HIT_CRASH: usize = 72;
pub const CAROUSEL_HIT_COWBELL: usize = 73;
pub const CAROUSEL_HIT_TAMB: usize = 74;
pub const CAROUSEL_HIT_WOOD: usize = 75;

/// Half-thickness of each instrument measured along the arm axis (used for top-height correction).
fn instrument_top_half(slot: u8) -> f32 {
    match slot {
        0 => CAROUSEL_CRASH_HALF_HEIGHT,
        1 => CAROUSEL_COWBELL_HALF_Y,
        2 => CAROUSEL_TAMB_HALF_HEIGHT,
        _ => CAROUSEL_WOOD_HALF_Y,
    }
}

/// Effective arm radius for a slot so its top face stays at a constant world height
/// regardless of which slot is on top or what tilt is applied.
pub fn slot_arm_radius(slot: u8, tilt_deg: f32) -> f32 {
    CAROUSEL_ARM_RADIUS - instrument_top_half(slot) * tilt_deg.to_radians().cos()
}

/// World Transform for a carousel instrument slot at the given carousel angle.
/// The tilt rotates the instrument away from the arm axis so marbles bounce sideways.
pub fn slot_transform(params: &CarouselParams, slot: u8, carousel_angle: f32) -> Transform {
    let arm_angle = carousel_angle + slot as f32 * std::f32::consts::FRAC_PI_2;
    let r = slot_arm_radius(slot, params.tilt_deg);
    let tilt = Quat::from_rotation_z(params.tilt_deg.to_radians());
    Transform {
        translation: params.pos + Vec3::new(0.0, r * arm_angle.cos(), r * arm_angle.sin()),
        // Arm rotation positions the slot; tilt is then applied in the instrument's local space.
        rotation: Quat::from_rotation_x(arm_angle) * tilt,
        scale: Vec3::ONE,
    }
}

/// World Transform for the visual arm of a slot — always points straight along the arm,
/// with no instrument tilt applied. Y-scale stretches the tube to reach the instrument.
pub fn arm_transform(params: &CarouselParams, slot: u8, carousel_angle: f32) -> Transform {
    let arm_angle = carousel_angle + slot as f32 * std::f32::consts::FRAC_PI_2;
    let r = slot_arm_radius(slot, params.tilt_deg);
    let half = r * 0.5;
    Transform {
        translation: params.pos + Vec3::new(0.0, half * arm_angle.cos(), half * arm_angle.sin()),
        rotation: Quat::from_rotation_x(arm_angle),
        // Y-scale adjusts effective arm length; the mesh was built for CAROUSEL_ARM_RADIUS.
        scale: Vec3::new(1.0, r / CAROUSEL_ARM_RADIUS, 1.0),
    }
}

pub fn spawn_carousel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    params: &CarouselParams,
) {
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(DARK_STEEL_COLOR.0, DARK_STEEL_COLOR.1, DARK_STEEL_COLOR.2),
        metallic: DARK_STEEL_METALLIC,
        perceptual_roughness: DARK_STEEL_ROUGHNESS,
        ..default()
    });
    let crash_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(CAROUSEL_CRASH_COLOR.0, CAROUSEL_CRASH_COLOR.1, CAROUSEL_CRASH_COLOR.2),
        metallic: 0.90,
        perceptual_roughness: 0.15,
        ..default()
    });
    let cowbell_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(CAROUSEL_COWBELL_COLOR.0, CAROUSEL_COWBELL_COLOR.1, CAROUSEL_COWBELL_COLOR.2),
        metallic: 0.70,
        perceptual_roughness: 0.30,
        ..default()
    });
    let tamb_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(CAROUSEL_TAMB_COLOR.0, CAROUSEL_TAMB_COLOR.1, CAROUSEL_TAMB_COLOR.2),
        metallic: 0.10,
        perceptual_roughness: 0.65,
        ..default()
    });
    let wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(CAROUSEL_WOOD_COLOR.0, CAROUSEL_WOOD_COLOR.1, CAROUSEL_WOOD_COLOR.2),
        metallic: 0.05,
        perceptual_roughness: 0.80,
        ..default()
    });

    // Shared arm-tube mesh.
    let arm_mesh = meshes.add(Cylinder {
        radius: CAROUSEL_ARM_TUBE_RADIUS,
        half_height: CAROUSEL_ARM_RADIUS * 0.5,
    });

    // Central axis cylinder — visual only, no physics. Rotated so its length runs along X.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder {
            radius: CAROUSEL_BODY_RADIUS,
            half_height: CAROUSEL_BODY_HALF_HEIGHT,
        })),
        MeshMaterial3d(frame_mat.clone()),
        Transform {
            translation: params.pos,
            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            scale: Vec3::ONE,
        },
        CarouselPart,
        CarouselBody,
    ));

    // Spawn 4 arms + instruments.
    let slots: [(u8, usize, Handle<StandardMaterial>); 4] = [
        (0, CAROUSEL_HIT_CRASH,  crash_mat),
        (1, CAROUSEL_HIT_COWBELL, cowbell_mat),
        (2, CAROUSEL_HIT_TAMB,   tamb_mat),
        (3, CAROUSEL_HIT_WOOD,   wood_mat),
    ];

    for (slot, hit_ch, mat) in slots {
        // Visual arm tube.
        commands.spawn((
            Mesh3d(arm_mesh.clone()),
            MeshMaterial3d(frame_mat.clone()),
            arm_transform(params, slot, 0.0),
            CarouselPart,
            CarouselArm(slot),
        ));

        // Instrument body + physics.
        match slot {
            0 => {
                // Crash cymbal — large thin disc.
                commands.spawn((
                    Mesh3d(meshes.add(Cylinder {
                        radius: CAROUSEL_CRASH_RADIUS,
                        half_height: CAROUSEL_CRASH_HALF_HEIGHT,
                    })),
                    MeshMaterial3d(mat),
                    slot_transform(params, slot, 0.0),
                    RigidBody::Static,
                    Collider::cylinder(CAROUSEL_CRASH_RADIUS, CAROUSEL_CRASH_HALF_HEIGHT * 2.0),
                    CollisionLayers::new(GameLayer::Default, [GameLayer::Marble]),
                    Restitution::new(params.crash_restitution),
                    Friction::new(params.crash_friction),
                    CollisionEventsEnabled,
                    Instrument { channel: hit_ch },
                    CarouselPart,
                    CarouselSlot(slot),
                ));
            }
            1 => {
                // Cowbell — rectangular block.
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid {
                        half_size: Vec3::new(CAROUSEL_COWBELL_HALF_X, CAROUSEL_COWBELL_HALF_Y, CAROUSEL_COWBELL_HALF_Z),
                    })),
                    MeshMaterial3d(mat),
                    slot_transform(params, slot, 0.0),
                    RigidBody::Static,
                    Collider::cuboid(CAROUSEL_COWBELL_HALF_X, CAROUSEL_COWBELL_HALF_Y, CAROUSEL_COWBELL_HALF_Z),
                    CollisionLayers::new(GameLayer::Default, [GameLayer::Marble]),
                    Restitution::new(params.cowbell_restitution),
                    Friction::new(params.cowbell_friction),
                    CollisionEventsEnabled,
                    Instrument { channel: hit_ch },
                    CarouselPart,
                    CarouselSlot(slot),
                ));
            }
            2 => {
                // Tambourine — thin disc.
                commands.spawn((
                    Mesh3d(meshes.add(Cylinder {
                        radius: CAROUSEL_TAMB_RADIUS,
                        half_height: CAROUSEL_TAMB_HALF_HEIGHT,
                    })),
                    MeshMaterial3d(mat),
                    slot_transform(params, slot, 0.0),
                    RigidBody::Static,
                    Collider::cylinder(CAROUSEL_TAMB_RADIUS, CAROUSEL_TAMB_HALF_HEIGHT * 2.0),
                    CollisionLayers::new(GameLayer::Default, [GameLayer::Marble]),
                    Restitution::new(params.tamb_restitution),
                    Friction::new(params.tamb_friction),
                    CollisionEventsEnabled,
                    Instrument { channel: hit_ch },
                    CarouselPart,
                    CarouselSlot(slot),
                ));
            }
            _ => {
                // Woodblock — small rectangular block.
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid {
                        half_size: Vec3::new(CAROUSEL_WOOD_HALF_X, CAROUSEL_WOOD_HALF_Y, CAROUSEL_WOOD_HALF_Z),
                    })),
                    MeshMaterial3d(mat),
                    slot_transform(params, slot, 0.0),
                    RigidBody::Static,
                    Collider::cuboid(CAROUSEL_WOOD_HALF_X, CAROUSEL_WOOD_HALF_Y, CAROUSEL_WOOD_HALF_Z),
                    CollisionLayers::new(GameLayer::Default, [GameLayer::Marble]),
                    Restitution::new(params.wood_restitution),
                    Friction::new(params.wood_friction),
                    CollisionEventsEnabled,
                    Instrument { channel: hit_ch },
                    CarouselPart,
                    CarouselSlot(slot),
                ));
            }
        }
    }
}
