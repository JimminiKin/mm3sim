// All tunable physics/geometry parameters exposed in the Parameters panel.
// Paste the output of "Copy params as consts" to fully replace this file.

// ── Ghost Snare ───────────────────────────────────────────────────────────────
pub const CHUTE_EXIT_Z: f32 = 0.27;
pub const CHUTE_EXIT_Y: f32 = 0.119;
pub const CHUTE_EXIT_LENGTH: f32 = 0.065;
pub const CHUTE_EXIT_ANGLE: f32 = 30.0;
pub const CHUTE_CURVE_RADIUS: f32 = 0.15;
pub const CHUTE_SLOPE_ANGLE: f32 = 82.0;
pub const CHUTE_SLOPE_LENGTH: f32 = 0.19;
pub const CHUTE_RESTITUTION: f32 = 0.05;
pub const CHUTE_FRICTION: f32 = 0.2;
pub const CHUTE_ANGLES: [f32; 6] = [-5.0, -8.0, -11.0, -14.0, -17.0, -20.0];

// ── Snare ─────────────────────────────────────────────────────────────────────
pub const SNARE_RESTITUTION: f32 = 0.6;
pub const SNARE_FRICTION: f32 = 0.18;
pub const SNARE_POS_X: f32 = 0.015;
pub const SNARE_POS_Y: f32 = 0.001;
pub const SNARE_POS_Z: f32 = -0.01;

// ── Vibraphone ────────────────────────────────────────────────────────────────
pub const VIB_ROW_X: f32 = 0.0;
pub const VIB_ROW_Y: f32 = -0.2;
pub const VIB_ROW_Z: f32 = -0.51;
pub const VIB_BAR_WIDTH: f32 = 0.045;
pub const VIB_BAR_SPACING: f32 = 0.055;
pub const VIB_BAR_THICKNESS: f32 = 0.01;
pub const VIB_BAR_LENGTH_MAX: f32 = 0.39;
pub const VIB_BAR_LENGTH_MIN: f32 = 0.14;
pub const VIB_BAR_DENSITY: f32 = 2700.0;
pub const VIB_ANGULAR_DAMPING: f32 = 0.3;
pub const VIB_RESTITUTION: f32 = 0.5;
pub const VIB_FRICTION: f32 = 0.15;
pub const VIB_ARM_SCALE: f32 = 0.83;
pub const VIB_PIVOT_FRAC: f32 = 0.276;
pub const VIB_REST_DEG: f32 = 10.0;
pub const VIB_MAX_TILT_DEG: f32 = 5.0;
pub const VIB_CW_WEIGHT_RATIO: f32 = 1.07;

// ── Hi-hat ────────────────────────────────────────────────────────────────────
pub const HIHAT_X: f32 = 0.265;
pub const HIHAT_Y: f32 = 0.12;
pub const HIHAT_Z: f32 = 0.153;
pub const HIHAT_RESTITUTION: f32 = 0.55;
pub const HIHAT_FRICTION: f32 = 0.15;
pub const HIHAT_GAP_OPEN: f32 = 0.025;
pub const HIHAT_GAP_CLOSED: f32 = 0.004;

// ── Kick ──────────────────────────────────────────────────────────────────────
pub const KICK_X: f32 = 0.625;
pub const KICK_Y: f32 = -0.151;
pub const KICK_Z: f32 = 0.076;
pub const KICK_RESTITUTION: f32 = 0.35;
pub const KICK_FRICTION: f32 = 0.25;
pub const KICK_REST_DEG: f32 = 15.0;
pub const KICK_MAX_TILT_DEG: f32 = 2.0;
pub const KICK_ANGULAR_DAMPING: f32 = 0.0;
pub const KICK_CW_WEIGHT_RATIO: f32 = 1.07;

// ── Ride ──────────────────────────────────────────────────────────────────────
pub const RIDE_X: f32 = 0.896;
pub const RIDE_Y: f32 = 0.123;
pub const RIDE_Z: f32 = 0.31;
pub const RIDE_RESTITUTION: f32 = 0.55;
pub const RIDE_FRICTION: f32 = 0.15;
