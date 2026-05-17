// ── World ────────────────────────────────────────────────────────────────────
pub const BG_COLOR: (f32, f32, f32) = (0.05, 0.05, 0.08);

// ── Lighting ─────────────────────────────────────────────────────────────────
pub const LIGHT_ILLUMINANCE: f32 = 25_000.0;
pub const LIGHT_ROT_X: f32 = -0.9;
pub const LIGHT_ROT_Y: f32 = 0.7;
pub const LIGHT_ROT_Z: f32 = 0.0;
pub const AMBIENT_BRIGHTNESS: f32 = 0.35;

// ── Camera ───────────────────────────────────────────────────────────────────
pub const CAMERA_POS: (f32, f32, f32) = (0.0, 8.0, 14.0);
// Derived from CAMERA_POS: radius = sqrt(8²+14²), pitch = -atan(8/14)
pub const CAMERA_INITIAL_RADIUS: f32 = 16.12;
pub const CAMERA_INITIAL_PITCH: f32 = -0.52;
pub const CAMERA_INITIAL_YAW: f32 = 0.0;
pub const CAMERA_ORBIT_SENSITIVITY: f32 = 0.005;
pub const CAMERA_PAN_SENSITIVITY: f32 = 0.0015;
pub const CAMERA_ZOOM_SPEED: f32 = 0.8;
pub const CAMERA_SCROLL_PIXEL_FACTOR: f32 = 0.1;
pub const CAMERA_PITCH_MIN: f32 = -1.5;
pub const CAMERA_PITCH_MAX: f32 = 0.4;
pub const CAMERA_RADIUS_MIN: f32 = 3.0;
pub const CAMERA_RADIUS_MAX: f32 = 60.0;

// ── Physics ───────────────────────────────────────────────────────────────────
pub const STEEL_RESTITUTION: f32 = 0.60;
pub const STEEL_FRICTION: f32 = 0.18;

// ── Snare drum ────────────────────────────────────────────────────────────────
pub const SNARE_RADIUS: f32 = 1.775; // 14" diameter
pub const SNARE_HALF_HEIGHT: f32 = 0.70; // 5.5" depth
pub const SNARE_MASS: f32 = 4.0; // kg

// ── Pivot arm ─────────────────────────────────────────────────────────────────
pub const ARM_LENGTH: f32 = 8.0; // 80 cm
pub const ARM_TUBE_RADIUS: f32 = 0.025; // 2.5 cm
pub const PIVOT_TO_EDGE_GAP: f32 = 2.0; // 20 cm from snare edge to pivot
pub const ARM_LINEAR_DAMPING: f32 = 0.0;
pub const ARM_ANGULAR_DAMPING: f32 = 0.0;
pub const PIVOT_STAND_HALF_HEIGHT: f32 = 0.5;

// Derived arm geometry (all relative to world origin = snare centre)
pub const PIVOT_FROM_SNARE: f32 = SNARE_RADIUS + PIVOT_TO_EDGE_GAP;
pub const ARM_HALF_LEN: f32 = ARM_LENGTH / 2.0;
pub const ARM_CENTER_Z: f32 = ARM_HALF_LEN;
pub const SNARE_LOCAL_Z: f32 = -ARM_HALF_LEN;
pub const PIVOT_LOCAL_Z: f32 = PIVOT_FROM_SNARE - ARM_CENTER_Z;
pub const CW_LOCAL_Z: f32 = ARM_HALF_LEN;

// Counterweight: mass computed so torques balance about the pivot
pub const CW_DISTANCE: f32 = ARM_LENGTH - PIVOT_FROM_SNARE;
pub const CW_RATIO: f32 = 1.1;
pub const CW_MASS: f32 = SNARE_MASS * PIVOT_FROM_SNARE / CW_DISTANCE * CW_RATIO;
pub const CW_RADIUS: f32 = 0.12;
pub const CW_HALF_HEIGHT: f32 = 0.08;

// Arm spawn angle (negative = snare-side down)
pub const ARM_SPAWN_DEG: f32 = -16.0;

// ── Marble ────────────────────────────────────────────────────────────────────
pub const MARBLE_RADIUS: f32 = 0.10;
pub const MARBLE_MASS: f32 = 0.033; // kg — steel at 20 mm diameter
pub const MARBLE_SPAWN_X: f32 = 0.0; // centre of snare
pub const SPAWN_HEIGHT: f32 = 8.0;
pub const MARBLE_SPAWN_JITTER: f32 = 0.001;
pub const DESPAWN_Y: f32 = -10.0;

// ── Materials ─────────────────────────────────────────────────────────────────
pub const CHROME_COLOR: (f32, f32, f32) = (0.75, 0.75, 0.80);
pub const CHROME_METALLIC: f32 = 0.95;
pub const CHROME_ROUGHNESS: f32 = 0.10;

pub const DARK_STEEL_COLOR: (f32, f32, f32) = (0.30, 0.30, 0.35);
pub const DARK_STEEL_METALLIC: f32 = 0.90;
pub const DARK_STEEL_ROUGHNESS: f32 = 0.20;

pub const MARBLE_COLOR: (f32, f32, f32) = (0.95, 0.35, 0.15);
pub const MARBLE_METALLIC: f32 = 0.80;
pub const MARBLE_ROUGHNESS: f32 = 0.20;

// ── Pivot stop ────────────────────────────────────────────────────────────────
// Contact point is just past the snare edge so the stop post never overlaps the snare collider
pub const STOP_CONTACT_Z_REST: f32 = 2.5;
pub const STOP_ARM_DIST: f32 = PIVOT_FROM_SNARE - STOP_CONTACT_Z_REST;
// Both stops are slender tubes running along X (perpendicular to the arm)
pub const STOP_TUBE_RADIUS: f32 = 0.03;
pub const STOP_TUBE_HALF_LEN: f32 = 0.30;
// Arm angle at which each stop is contacted (positive = snare-side down)
pub const STOP_LOWER_DEG: f32 = 17.0;
pub const STOP_UPPER_DEG: f32 = 15.0;

// ── Chute ─────────────────────────────────────────────────────────────────────
// Cubic Bézier profile in the Y-Z plane: P0 → CP1 → CP2 → P3
// P0 = (CHUTE_START_Z, CHUTE_START_Y), P3 = (CHUTE_END_Z, CHUTE_END_Y)
pub const CHUTE_END_X: f32 = 0.0;
pub const CHUTE_END_Y: f32 = 2.0;
pub const CHUTE_END_Z: f32 = 0.0;
pub const CHUTE_START_Z: f32 = 3.83;
pub const CHUTE_START_Y: f32 = 4.44;
pub const CHUTE_CP1: (f32, f32) = (3.83, 2.0);  // (z, y) first inner handle
pub const CHUTE_CP2: (f32, f32) = (0.0, 3.5);   // (z, y) second inner handle
pub const CHUTE_THICKNESS: f32 = 0.04;
pub const CHUTE_WIDTH: f32 = 0.30;
pub const CHUTE_SEGMENTS: usize = 32;
pub const CHUTE_MARBLE_COLOR: (f32, f32, f32) = (0.20, 0.45, 0.90);
