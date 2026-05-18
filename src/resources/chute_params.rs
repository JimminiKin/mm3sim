use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum DragAxis {
    #[default]
    Free,
    Vertical,   // Y only
    Horizontal, // Z only
}

#[derive(Resource)]
pub struct ChuteParams {
    pub p0: [f32; 2],   // start (z, y)
    pub cp1: [f32; 2],  // first inner handle (z, y)
    pub cp2: [f32; 2],  // second inner handle (z, y)
    pub p3: [f32; 2],   // end (z, y)
    pub straight: bool,
    pub handles_visible: bool,
    pub endpoints_visible: bool,
    pub drag_axis: DragAxis,
    pub dirty: bool,
}

impl Default for ChuteParams {
    fn default() -> Self {
        Self {
            p0:  [CHUTE_START_Z, CHUTE_START_Y],
            cp1: [CHUTE_CP1.0,   CHUTE_CP1.1],
            cp2: [CHUTE_CP2.0,   CHUTE_CP2.1],
            p3:  [CHUTE_END_Z,   CHUTE_END_Y],
            straight: true,
            handles_visible: true,
            endpoints_visible: true,
            drag_axis: DragAxis::Free,
            dirty: false,
        }
    }
}

impl ChuteParams {
    /// Returns the four control points to actually build the curve.
    /// In straight mode CP1 and CP2 are placed at 1/3 and 2/3 along P0→P3,
    /// making the Bézier degenerate to a straight line.
    pub fn effective_pts(&self) -> [[f32; 2]; 4] {
        if self.straight {
            let [z0, y0] = self.p0;
            let [z3, y3] = self.p3;
            let cp1 = [z0 + (z3 - z0) / 3.0, y0 + (y3 - y0) / 3.0];
            let cp2 = [z0 + (z3 - z0) * 2.0 / 3.0, y0 + (y3 - y0) * 2.0 / 3.0];
            [self.p0, cp1, cp2, self.p3]
        } else {
            [self.p0, self.cp1, self.cp2, self.p3]
        }
    }
}
