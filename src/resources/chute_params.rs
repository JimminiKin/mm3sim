use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum DragAxis {
    #[default]
    Free,
    Vertical,   // Y only
    Horizontal, // Z only
}

/// Derived geometry for the 3-part chute (computed from ChuteParams).
pub struct ChuteGeometry {
    pub slope_start: [f32; 2],    // [z, y]
    pub arc_start: [f32; 2],      // [z, y] where slope meets arc
    pub center: [f32; 2],         // [z, y] center of circular arc
    pub exit_start: [f32; 2],     // [z, y] where arc meets exit section
    pub slope_tangent: [f32; 2],  // [tz, ty] forward direction along slope
    pub exit_tangent: [f32; 2],   // [tz, ty] forward direction along exit
    pub theta_start: f32,         // angle of arc_start from center (radians)
    pub arc_sweep: f32,           // arc angular sweep (negative = CW)
}

#[derive(Resource)]
pub struct ChuteParams {
    pub exit_pos: [f32; 2],    // [z, y] exit end point (near snare)
    pub exit_length: f32,      // length of straight exit section
    pub exit_angle: f32,       // degrees below horizontal (usually 0)
    pub curve_radius: f32,     // radius of circular transition arc
    pub slope_angle: f32,      // degrees below horizontal (must exceed exit_angle)
    pub slope_length: f32,     // length of entry slope
    pub drag_axis: DragAxis,
    pub handles_visible: bool,
    pub dirty: bool,
}

impl Default for ChuteParams {
    fn default() -> Self {
        Self {
            exit_pos: [CHUTE_EXIT_Z, CHUTE_EXIT_Y],
            exit_length: CHUTE_EXIT_LENGTH,
            exit_angle: CHUTE_EXIT_ANGLE,
            curve_radius: CHUTE_CURVE_RADIUS,
            slope_angle: CHUTE_SLOPE_ANGLE,
            slope_length: CHUTE_SLOPE_LENGTH,
            drag_axis: DragAxis::Free,
            handles_visible: false,
            dirty: false,
        }
    }
}

impl ChuteParams {
    /// Computes all derived geometry from the current parameters.
    ///
    /// Convention: marble travels in the (-Z, -Y) direction (toward snare, downward).
    /// Forward tangent = (-cos(angle), -sin(angle)) in (Z, Y) space.
    /// Inward normal (toward arc center) = CW rotation of tangent = (ty, -tz).
    pub fn geometry(&self) -> ChuteGeometry {
        // Clamp so slope is always steeper than exit (minimum 1° difference)
        let ea = self.exit_angle.to_radians();
        let sa = self.slope_angle.to_radians().max(ea + 1f32.to_radians());

        // Forward tangents (marble travel direction toward snare)
        let exit_tz = -ea.cos();
        let exit_ty = -ea.sin();
        let slope_tz = -sa.cos();
        let slope_ty = -sa.sin();

        let (exit_z, exit_y) = (self.exit_pos[0], self.exit_pos[1]);

        // Exit section: exit_start → exit_end
        let exit_start_z = exit_z - self.exit_length * exit_tz;
        let exit_start_y = exit_y - self.exit_length * exit_ty;

        // Inward normal = CW rotation of tangent: CW(tz, ty) = (ty, -tz)
        let n_ez = exit_ty;
        let n_ey = -exit_tz;

        // Arc center (exit_length above exit_start, perpendicular to exit)
        let center_z = exit_start_z + self.curve_radius * n_ez;
        let center_y = exit_start_y + self.curve_radius * n_ey;

        // Slope inward normal
        let n_sz = slope_ty;
        let n_sy = -slope_tz;

        // Arc start (where slope tangentially meets the arc)
        let arc_start_z = center_z - self.curve_radius * n_sz;
        let arc_start_y = center_y - self.curve_radius * n_sy;

        // Slope start
        let slope_start_z = arc_start_z - self.slope_length * slope_tz;
        let slope_start_y = arc_start_y - self.slope_length * slope_ty;

        // Arc angular parameters (CW sweep from arc_start to arc_end/exit_start)
        let theta_start = f32::atan2(arc_start_y - center_y, arc_start_z - center_z);
        let arc_sweep = -(sa - ea); // negative = clockwise

        ChuteGeometry {
            slope_start: [slope_start_z, slope_start_y],
            arc_start: [arc_start_z, arc_start_y],
            center: [center_z, center_y],
            exit_start: [exit_start_z, exit_start_y],
            slope_tangent: [slope_tz, slope_ty],
            exit_tangent: [exit_tz, exit_ty],
            theta_start,
            arc_sweep,
        }
    }
}
