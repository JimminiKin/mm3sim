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
pub const VIB_MARBLE_COLOR:   (f32, f32, f32) = (0.20, 0.80, 0.35);

// =============================================================================
// Physics & Geometry
// =============================================================================

// ── Simulation ────────────────────────────────────────────────────────────────
pub const SIMULATION_TPS: f32 = 1000.0;

// ── Surface physics ───────────────────────────────────────────────────────────
pub const STEEL_RESTITUTION: f32 = 0.60; // marble
pub const STEEL_FRICTION: f32 = 0.18; // marble
pub const CHUTE_RESTITUTION: f32 = 0.05;
pub const CHUTE_FRICTION: f32 = 0.20; // ABS has moderate grip on steel
pub const SNARE_RESTITUTION: f32 = 0.60;
pub const SNARE_FRICTION: f32 = 0.18;

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
pub const CW_WEIGHT_RATIO: f32 = 1.070; // fraction above torque balance; > 0 → arm rests at upper joint limit
pub const CW_MASS: f32 =
    (SNARE_MASS * PIVOT_FROM_SNARE + ARM_MASS * PIVOT_LOCAL_Z) / CW_DISTANCE * (CW_WEIGHT_RATIO);
pub const CW_RADIUS: f32 = 0.02;
pub const CW_HALF_HEIGHT: f32 = 0.08;

// ── Pivot joint limits ────────────────────────────────────────────────────────
pub const SNARE_REST_DEG: f32 = 15.0; // snare tilt at rest (positive = snare-side down)
pub const MAX_TILT_DEG: f32 = 2.0; // max additional downward tilt from rest on impact

// Arm spawns at its rest angle (upper joint limit)
pub const ARM_SPAWN_DEG: f32 = -SNARE_REST_DEG;

// ── Marble ────────────────────────────────────────────────────────────────────
pub const MARBLE_RADIUS: f32 = 0.0075;
pub const MARBLE_MASS: f32 = 0.014; // kg — steel at 20 mm diameter
pub const SPAWN_HEIGHT: f32 = 1.0; // above snare top face
pub const DROP_REFERENCE_S: f32 = 0.450; // theoretical 1 m free-fall flight time
/// XZ jitter applied at marble spawn (realistic release noise, in metres).
pub const MARBLE_SPAWN_JITTER: f32 = 0.001;
pub const DESPAWN_Y: f32 = -0.3;
pub const CHUTE_MARBLE_DESPAWN_Y: f32 = -0.1; // chute marbles exit near snare height; cull sooner

// ── Vibraphone ────────────────────────────────────────────────────────────────
pub const VIB_BAR_COUNT: u32 = 37;
pub const VIB_BAR_WIDTH: f32 = 0.045;
pub const VIB_BAR_SPACING: f32 = 0.055;
pub const VIB_BAR_THICKNESS: f32 = 0.010;
pub const VIB_BAR_LENGTH_MAX: f32 = 0.390;
pub const VIB_BAR_LENGTH_MIN: f32 = 0.140;
pub const VIB_ROW_Z: f32 = -0.51;
pub const VIB_ROW_Y: f32 = -0.2; // top face Y (snare top = 0.070, slightly lower)

pub const VIB_RESTITUTION: f32 = 0.50;
pub const VIB_FRICTION: f32 = 0.15;
pub const VIB_BAR_DENSITY: f32 = 2700.0; // aluminium alloy, kg/m³

// arm_length = bar_length * arm_scale; pivot = bar_length * pivot_frac from bar center toward CW
pub const VIB_ARM_SCALE: f32 = 0.83;
pub const VIB_PIVOT_FRAC: f32 = 0.276; // resonance node: 22.4% from far end = 27.6% from center
pub const VIB_ARM_TUBE_RADIUS: f32 = 0.003;
pub const VIB_ARM_MASS: f32 = 0.05;

pub const VIB_REST_DEG: f32 = 10.0;
pub const VIB_MAX_TILT_DEG: f32 = 5.0;
pub const VIB_LINEAR_DAMPING: f32 = 0.0;
pub const VIB_ANGULAR_DAMPING: f32 = 0.3;
pub const VIB_CW_WEIGHT_RATIO: f32 = 1.07;
pub const VIB_CW_RADIUS: f32 = 0.012;
pub const VIB_CW_HALF_HEIGHT: f32 = 0.018;

pub const VIB_DROP_BAR_INDEX: u32 = 6;
pub const VIB_SPAWN_HEIGHT: f32 = 1.0; // height above bar top to spawn marble

pub const VIB_BAR_COLOR: (f32, f32, f32) = (0.82, 0.73, 0.33);
pub const VIB_BAR_METALLIC: f32 = 0.90;
pub const VIB_BAR_ROUGHNESS: f32 = 0.20;

// ── Chute ─────────────────────────────────────────────────────────────────────
// All Y/Z coords are relative to the snare top-face centre at arm θ=0.
// World position = param + CHUTE_ORIGIN_*.
pub const CHUTE_ORIGIN_Y: f32 = SNARE_HALF_HEIGHT; // snare top face above world origin
pub const CHUTE_ORIGIN_Z: f32 = 0.0; // snare centre is at z=0 when arm is level

// 3-part chute profile in the Y-Z plane: straight slope → circular arc → straight exit
// (0, 0) = centre of snare top face; positive Y = up, positive Z = away from snare
pub const CHUTE_END_X: f32 = 0.0;
pub const CHUTE_EXIT_Z: f32 = 0.27; // exit end point Z (near snare)
pub const CHUTE_EXIT_Y: f32 = 0.119; // exit end point Y (height)
pub const CHUTE_EXIT_LENGTH: f32 = 0.065; // length of horizontal exit section
pub const CHUTE_EXIT_ANGLE: f32 = 30.0; // degrees below horizontal (usually 0)
pub const CHUTE_CURVE_RADIUS: f32 = 0.150; // radius of transition arc
pub const CHUTE_SLOPE_ANGLE: f32 = 82.0; // degrees below horizontal
pub const CHUTE_SLOPE_LENGTH: f32 = 0.190; // length of entry slope
pub const CHUTE_THICKNESS: f32 = 0.01;
pub const CHUTE_WIDTH: f32 = 0.02;

// =============================================================================
// Programming Wheel
// =============================================================================

pub const PROGRAMMING_WHEEL_RADIUS: f32 = 0.5; // 1 m diameter cylinder
/// 16 bars × 4 beats/bar = 64 beats per revolution.
/// At 2.40625 RPM: 2.40625 × 64 = 154 musical BPM.
pub const PROGRAMMING_WHEEL_BEATS_PER_REV: f32 = 64.0;
/// ch 0–5 = chute channels (one per chute), ch 6–12 = snare variants, ch 13–49 = vib bars 0–36.
/// Must equal the number of entries in `CHANNEL_DEFS` in `programming_wheel_params`.
pub const PROGRAMMING_WHEEL_N_CHANNELS: usize = 50;
/// 154 BPM ÷ 64 beats/rev = 2.40625 RPM
pub const PROGRAMMING_WHEEL_RPM_DEFAULT: f32 = 2.40625;
pub const PROGRAMMING_WHEEL_Z_POS: f32 = 1.4; // world Z (positive from snare)
pub const PROGRAMMING_WHEEL_Y_POS: f32 = 0.8; // world Y (cylinder centre)
pub const PROGRAMMING_WHEEL_WIDTH: f32 = 2.2; // total X span of the wheel
pub const PROGRAMMING_WHEEL_READER_GAP: f32 = 0.014; // gap between cylinder surface and reader bar
pub const PROGRAMMING_WHEEL_READER_HALF_H: f32 = 0.012; // reader bar cross-section half-size
