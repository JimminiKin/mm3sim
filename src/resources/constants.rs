// Tunable parameters (physics, positions, surface properties) live in tuning.rs.
// Everything here is structural geometry, physics, or derived quantities that
// are not exposed in the Parameters panel.
pub use crate::resources::tuning::*;

// =============================================================================
// Camera & Scene
// =============================================================================

pub const BG_COLOR: (f32, f32, f32) = (0.05, 0.05, 0.08);

pub const LIGHT_ILLUMINANCE: f32 = 25_000.0;
pub const LIGHT_ROT_X: f32 = -0.9;
pub const LIGHT_ROT_Y: f32 = 0.7;
pub const LIGHT_ROT_Z: f32 = 0.0;
pub const AMBIENT_BRIGHTNESS: f32 = 0.35;

pub const CAMERA_INITIAL_RADIUS: f32 = 4.5;
pub const CAMERA_INITIAL_PITCH: f32 = -0.27;
pub const CAMERA_INITIAL_YAW: f32 = 2.10;
pub const CAMERA_ORBIT_SENSITIVITY: f32 = 0.005;
pub const CAMERA_PAN_SENSITIVITY: f32 = 0.0015;
pub const CAMERA_ZOOM_SPEED: f32 = 0.01;
pub const CAMERA_SCROLL_PIXEL_FACTOR: f32 = 0.1;
pub const CAMERA_PITCH_MIN: f32 = -1.5;
pub const CAMERA_PITCH_MAX: f32 = 0.4;
pub const CAMERA_RADIUS_MIN: f32 = 0.30;
pub const CAMERA_RADIUS_MAX: f32 = 6.0;

// =============================================================================
// Materials
// =============================================================================

pub const CHROME_COLOR: (f32, f32, f32) = (0.75, 0.75, 0.80);
pub const CHROME_METALLIC: f32 = 0.95;
pub const CHROME_ROUGHNESS: f32 = 0.10;

pub const DARK_STEEL_COLOR: (f32, f32, f32) = (0.30, 0.30, 0.35);
pub const DARK_STEEL_METALLIC: f32 = 0.90;
pub const DARK_STEEL_ROUGHNESS: f32 = 0.20;

pub const MARBLE_COLOR: (f32, f32, f32) = (0.95, 0.35, 0.15);
pub const MARBLE_METALLIC: f32 = 0.80;
pub const MARBLE_ROUGHNESS: f32 = 0.20;

pub const CHUTE_MARBLE_COLOR: (f32, f32, f32) = (0.20, 0.45, 0.90);
pub const VIB_MARBLE_COLOR: (f32, f32, f32) = (0.20, 0.80, 0.35);

// =============================================================================
// Physics & Geometry
// =============================================================================

// ── Simulation ────────────────────────────────────────────────────────────────
pub const SIMULATION_TPS: f32 = 1000.0;

// ── Surface physics (non-tunable) ────────────────────────────────────────────
pub const STEEL_RESTITUTION: f32 = 0.60; // marble on steel
pub const STEEL_FRICTION: f32 = 0.18;    // marble on steel

// ── Snare drum ────────────────────────────────────────────────────────────────
pub const SNARE_RADIUS: f32 = 0.1778; // 14" diameter
pub const SNARE_HALF_HEIGHT: f32 = 0.070; // 5.5" depth
pub const SNARE_MASS: f32 = 4.0; // kg

// ── Pivot arm ─────────────────────────────────────────────────────────────────
pub const ARM_LENGTH: f32 = 0.60; // 60 cm
pub const ARM_TUBE_RADIUS: f32 = 0.025; // 2.5 cm radius
pub const ARM_MASS: f32 = 1.0; // kg
pub const PIVOT_TO_EDGE_GAP: f32 = 0.20; // 20 cm from snare edge to pivot
pub const ARM_LINEAR_DAMPING: f32 = 0.0;
pub const ARM_ANGULAR_DAMPING: f32 = 0.0;

// Derived arm geometry (all relative to world origin = snare centre)
pub const PIVOT_FROM_SNARE: f32 = SNARE_RADIUS + PIVOT_TO_EDGE_GAP;
pub const ARM_HALF_LEN: f32 = ARM_LENGTH / 2.0;
pub const ARM_CENTER_Z: f32 = ARM_HALF_LEN;
pub const SNARE_LOCAL_Z: f32 = -ARM_HALF_LEN;
pub const PIVOT_LOCAL_Z: f32 = PIVOT_FROM_SNARE - ARM_CENTER_Z;
pub const CW_LOCAL_Z: f32 = ARM_HALF_LEN;

// ── Counterweight ─────────────────────────────────────────────────────────────
pub const CW_DISTANCE: f32 = ARM_LENGTH - PIVOT_FROM_SNARE;
pub const CW_WEIGHT_RATIO: f32 = 1.070;
pub const CW_MASS: f32 =
    (SNARE_MASS * PIVOT_FROM_SNARE + ARM_MASS * PIVOT_LOCAL_Z) / CW_DISTANCE * (CW_WEIGHT_RATIO);
pub const CW_RADIUS: f32 = 0.02;
pub const CW_HALF_HEIGHT: f32 = 0.08;

// ── Pivot joint limits ────────────────────────────────────────────────────────
pub const SNARE_REST_DEG: f32 = 15.0;
pub const MAX_TILT_DEG: f32 = 2.0;
pub const ARM_SPAWN_DEG: f32 = -SNARE_REST_DEG;

// ── Marble ────────────────────────────────────────────────────────────────────
pub const MARBLE_RADIUS: f32 = 0.0075;
pub const MARBLE_MASS: f32 = 0.014; // kg — steel at 20 mm diameter
pub const SPAWN_HEIGHT: f32 = 1.0; // above snare top face
pub const DROP_REFERENCE_S: f32 = 0.450; // theoretical 1 m free-fall flight time
pub const MARBLE_SPAWN_JITTER: f32 = 0.001;
pub const DESPAWN_Y: f32 = -0.3;
pub const BACKSIDE_INSTRUMENTS_MARBLE_DESPAWN_Y: f32 = -0.1;

// ── Vibraphone ────────────────────────────────────────────────────────────────
pub const VIB_BAR_COUNT: u32 = 37;
pub const VIB_ARM_TUBE_RADIUS: f32 = 0.003;
pub const VIB_ARM_MASS: f32 = 0.05;
pub const VIB_LINEAR_DAMPING: f32 = 0.0;
pub const VIB_CW_RADIUS: f32 = 0.012;
pub const VIB_CW_HALF_HEIGHT: f32 = 0.018;
pub const VIB_SPAWN_HEIGHT: f32 = 1.0;

pub const VIB_BAR_COLOR: (f32, f32, f32) = (0.82, 0.73, 0.33);
pub const VIB_BAR_METALLIC: f32 = 0.90;
pub const VIB_BAR_ROUGHNESS: f32 = 0.20;

// ── Chute (structural geometry) ───────────────────────────────────────────────
pub const CHUTE_ORIGIN_Y: f32 = SNARE_HALF_HEIGHT;
pub const CHUTE_ORIGIN_Z: f32 = 0.0;
pub const CHUTE_END_X: f32 = 0.0;
pub const CHUTE_THICKNESS: f32 = 0.01;
pub const CHUTE_WIDTH: f32 = 0.02;

// =============================================================================
// Programming Wheel
// =============================================================================

pub const PROGRAMMING_WHEEL_RADIUS: f32 = 0.5;
pub const PROGRAMMING_WHEEL_BEATS_PER_REV: f32 = 64.0;
pub const PROGRAMMING_WHEEL_N_CHANNELS: usize = 69;
pub const PROGRAMMING_WHEEL_RPM_DEFAULT: f32 = 2.40625;
pub const PROGRAMMING_WHEEL_Z_POS: f32 = 1.4;
pub const PROGRAMMING_WHEEL_Y_POS: f32 = 0.8;
pub const PROGRAMMING_WHEEL_WIDTH: f32 = 2.2;
pub const PROGRAMMING_WHEEL_READER_GAP: f32 = 0.014;
pub const PROGRAMMING_WHEEL_READER_HALF_H: f32 = 0.012;

// =============================================================================
// Hi-hat (structural geometry)
// =============================================================================

pub const HIHAT_RADIUS: f32 = 0.15; // ~12" cymbal
pub const HIHAT_HALF_HEIGHT: f32 = 0.003; // 3 mm thick
pub const HIHAT_COLOR: (f32, f32, f32) = (0.72, 0.60, 0.18);
pub const HIHAT_METALLIC: f32 = 0.85;
pub const HIHAT_ROUGHNESS: f32 = 0.25;
pub const HIHAT_MARBLE_COLOR: (f32, f32, f32) = (0.95, 0.80, 0.15);

pub const HIHAT_PEDAL_HALF_HEIGHT: f32 = 0.003;

// =============================================================================
// Kick drum (structural geometry)
// =============================================================================

pub const KICK_RADIUS: f32 = 0.25; // ~20" diameter
pub const KICK_HALF_HEIGHT: f32 = 0.15; // ~12" depth
pub const KICK_COLOR: (f32, f32, f32) = (0.45, 0.28, 0.12);
pub const KICK_METALLIC: f32 = 0.05;
pub const KICK_ROUGHNESS: f32 = 0.75;
pub const KICK_MARBLE_COLOR: (f32, f32, f32) = (0.70, 0.45, 0.20);

// ── Kick drum (pivot arm geometry) ───────────────────────────────────────────
pub const KICK_MASS: f32 = 4.0;
pub const KICK_ARM_LENGTH: f32 = 1.0;
pub const KICK_ARM_HALF_LEN: f32 = KICK_ARM_LENGTH / 2.0;
pub const KICK_ARM_MASS: f32 = 1.0;
pub const KICK_ARM_TUBE_RADIUS: f32 = 0.025;
pub const KICK_PIVOT_FROM_DRUM: f32 = 0.45; // KICK_RADIUS + 0.20 gap
pub const KICK_PIVOT_LOCAL_Z: f32 = KICK_PIVOT_FROM_DRUM - KICK_ARM_HALF_LEN; // = -0.05
pub const KICK_CW_DISTANCE: f32 = KICK_ARM_HALF_LEN - KICK_PIVOT_LOCAL_Z;    // = 0.55
pub const KICK_CW_RADIUS: f32 = 0.03;
pub const KICK_CW_HALF_HEIGHT: f32 = 0.12;

// =============================================================================
// Ride cymbal (structural geometry)
// =============================================================================

pub const RIDE_RADIUS: f32 = 0.20; // ~16" diameter
pub const RIDE_HALF_HEIGHT: f32 = 0.003; // 3 mm thick
pub const RIDE_COLOR: (f32, f32, f32) = (0.78, 0.65, 0.25);
pub const RIDE_METALLIC: f32 = 0.88;
pub const RIDE_ROUGHNESS: f32 = 0.22;
pub const RIDE_MARBLE_COLOR: (f32, f32, f32) = (0.85, 0.70, 0.25);
