use bevy::prelude::*;

use crate::resources::constants::*;

/// World position offset applied to the entire snare mechanism.
///
/// `pos = Vec3::ZERO` reproduces the original hard-coded layout:
///   – pivot anchor at `(0, 0, PIVOT_FROM_SNARE)`
///   – snare drum centre at roughly `(0, −0.098, 0.013)` when at rest
///
/// Changing `pos` shifts the whole assembly (drum + arm + pivot + stand) by
/// the same vector.  Set `dirty = true` to trigger `rebuild_snare_system`.
#[derive(Resource, Debug, Clone)]
pub struct SnareParams {
    pub pos: Vec3,
    pub restitution: f32,
    pub friction: f32,
    pub dirty: bool,
}

impl Default for SnareParams {
    fn default() -> Self {
        Self {
            pos: Vec3::new(SNARE_POS_X, SNARE_POS_Y, SNARE_POS_Z),
            restitution: SNARE_RESTITUTION,
            friction: SNARE_FRICTION,
            dirty: false,
        }
    }
}
