use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Resource)]
pub struct ChuteParams {
    pub p0: [f32; 2],   // start (z, y)
    pub cp1: [f32; 2],  // first inner handle (z, y)
    pub cp2: [f32; 2],  // second inner handle (z, y)
    pub p3: [f32; 2],   // end (z, y)
    pub dirty: bool,
}

impl Default for ChuteParams {
    fn default() -> Self {
        Self {
            p0:  [CHUTE_START_Z, CHUTE_START_Y],
            cp1: [CHUTE_CP1.0,   CHUTE_CP1.1],
            cp2: [CHUTE_CP2.0,   CHUTE_CP2.1],
            p3:  [CHUTE_END_Z,   CHUTE_END_Y],
            dirty: false,
        }
    }
}
