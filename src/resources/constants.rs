// ── World ────────────────────────────────────────────────────────────────────
pub const BG_COLOR: (f32, f32, f32) = (0.05, 0.05, 0.08);

// ── Lighting ─────────────────────────────────────────────────────────────────
pub const LIGHT_ILLUMINANCE: f32 = 25_000.0;
pub const LIGHT_ROT_X: f32 = -0.9;
pub const LIGHT_ROT_Y: f32 = 0.7;
pub const LIGHT_ROT_Z: f32 = 0.0;
pub const AMBIENT_BRIGHTNESS: f32 = 0.35;

// ── Camera ───────────────────────────────────────────────────────────────────
pub const CAMERA_POS: (f32, f32, f32) = (1.6, -0.1, -0.18);
pub const CAMERA_INITIAL_RADIUS: f32 = 1.612;
pub const CAMERA_INITIAL_PITCH: f32 = -0.52;
pub const CAMERA_INITIAL_YAW: f32 = 1.7;
pub const CAMERA_ORBIT_SENSITIVITY: f32 = 0.005;
pub const CAMERA_PAN_SENSITIVITY: f32 = 0.0015;
pub const CAMERA_ZOOM_SPEED: f32 = 0.08;
pub const CAMERA_SCROLL_PIXEL_FACTOR: f32 = 0.1;
pub const CAMERA_PITCH_MIN: f32 = -1.5;
pub const CAMERA_PITCH_MAX: f32 = 0.4;
pub const CAMERA_RADIUS_MIN: f32 = 0.30;
pub const CAMERA_RADIUS_MAX: f32 = 6.0;

// ── Physics ───────────────────────────────────────────────────────────────────
pub const STEEL_RESTITUTION: f32 = 0.60;
pub const STEEL_FRICTION: f32 = 0.18;

// ── Snare drum ────────────────────────────────────────────────────────────────
pub const SNARE_RADIUS: f32 = 0.1778; // 14" diameter
pub const SNARE_HALF_HEIGHT: f32 = 0.070; // 5.5" depth
pub const SNARE_MASS: f32 = 4.0; // kg

// ── Pivot arm ─────────────────────────────────────────────────────────────────
pub const ARM_LENGTH: f32 = 0.80; // 80 cm
pub const ARM_TUBE_RADIUS: f32 = 0.025; // 2.5 cm radius
pub const PIVOT_TO_EDGE_GAP: f32 = 0.20; // 20 cm from snare edge to pivot
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
pub const ARM_SPAWN_DEG: f32 = -15.04;

// ── Marble ────────────────────────────────────────────────────────────────────
pub const MARBLE_RADIUS: f32 = 0.0075;
pub const MARBLE_MASS: f32 = 0.014; // kg — steel at 20 mm diameter
pub const MARBLE_SPAWN_X: f32 = 0.0; // centre of snare
pub const SPAWN_HEIGHT: f32 = 1.0; // above snare top centre
pub const MARBLE_SPAWN_JITTER: f32 = 0.001;
pub const DESPAWN_Y: f32 = -0.3;

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
pub const STOP_CONTACT_Z_REST: f32 = 0.25;
pub const STOP_ARM_DIST: f32 = PIVOT_FROM_SNARE - STOP_CONTACT_Z_REST;
// Both stops are slender tubes running along X (perpendicular to the arm)
pub const STOP_TUBE_RADIUS: f32 = 0.003;
pub const STOP_TUBE_HALF_LEN: f32 = 0.05;
// Arm angle at which each stop is contacted (positive = snare-side down)
pub const STOP_LOWER_DEG: f32 = 17.0;
pub const STOP_UPPER_DEG: f32 = 15.0;

// ── Chute ─────────────────────────────────────────────────────────────────────
// All chute Y/Z coords are relative to the snare top-face centre at arm θ=0.
// World position = param + CHUTE_ORIGIN_*.
pub const CHUTE_ORIGIN_Y: f32 = SNARE_HALF_HEIGHT; // snare top face above world origin
pub const CHUTE_ORIGIN_Z: f32 = 0.0; // snare centre is at z=0 when arm is level

// Cubic Bézier profile in the Y-Z plane: P0 → CP1 → CP2 → P3
// (0, 0) = centre of snare top face; positive Y = up, positive Z = away from snare
pub const CHUTE_END_X: f32 = 0.0;
pub const CHUTE_END_Y: f32 = 0.40;
pub const CHUTE_END_Z: f32 = 0.239;
pub const CHUTE_START_Z: f32 = 0.333;
pub const CHUTE_START_Y: f32 = 0.478;
pub const CHUTE_CP1: (f32, f32) = (0.323, 0.440); // (z, y) first inner handle
pub const CHUTE_CP2: (f32, f32) = (0.28, 0.400); // (z, y) second inner handle
pub const CHUTE_THICKNESS: f32 = 0.004;
pub const CHUTE_WIDTH: f32 = 0.30;
pub const CHUTE_RESTITUTION: f32 = 0.35;
pub const CHUTE_FRICTION: f32 = 0.20; // ABS has moderate grip on steel
pub const CHUTE_MARBLE_COLOR: (f32, f32, f32) = (0.20, 0.45, 0.90);
