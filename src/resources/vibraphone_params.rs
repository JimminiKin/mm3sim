use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Resource)]
pub struct VibraphoneParams {
    // Row position
    pub row_z: f32,
    pub row_y: f32,
    pub row_x_center: f32,

    // Bar geometry
    pub bar_width: f32,
    pub bar_spacing: f32,
    pub bar_thickness: f32,
    pub bar_length_max: f32,
    pub bar_length_min: f32,

    // Bar physics
    pub restitution: f32,
    pub friction: f32,
    pub bar_density: f32, // kg/m³ — mass computed as density × width × thickness × length per bar
    pub angular_damping: f32,

    // Pivot arm (dimensions scale with bar_length)
    pub arm_scale: f32,    // arm_length = bar_length * arm_scale
    pub pivot_frac: f32,   // pivot_from_bar = bar_length * pivot_frac (from bar center toward CW)
    pub rest_deg: f32,
    pub max_tilt_deg: f32,
    pub cw_weight_ratio: f32,

    // Marble drop
    pub drop_bar_index: u32,
    pub spawn_marble: bool,

    pub dirty: bool,
}

impl Default for VibraphoneParams {
    fn default() -> Self {
        Self {
            row_z: VIB_ROW_Z,
            row_y: VIB_ROW_Y,
            row_x_center: 0.0,
            bar_width: VIB_BAR_WIDTH,
            bar_spacing: VIB_BAR_SPACING,
            bar_thickness: VIB_BAR_THICKNESS,
            bar_length_max: VIB_BAR_LENGTH_MAX,
            bar_length_min: VIB_BAR_LENGTH_MIN,
            restitution: VIB_RESTITUTION,
            friction: VIB_FRICTION,
            bar_density: VIB_BAR_DENSITY,
            angular_damping: VIB_ANGULAR_DAMPING,
            arm_scale: VIB_ARM_SCALE,
            pivot_frac: VIB_PIVOT_FRAC,
            rest_deg: VIB_REST_DEG,
            max_tilt_deg: VIB_MAX_TILT_DEG,
            cw_weight_ratio: VIB_CW_WEIGHT_RATIO,
            drop_bar_index: VIB_DROP_BAR_INDEX,
            spawn_marble: true,
            dirty: false,
        }
    }
}
