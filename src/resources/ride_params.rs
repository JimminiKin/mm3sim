use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Resource)]
pub struct RideParams {
    pub pos: Vec3,
    pub restitution: f32,
    pub friction: f32,
    pub dirty: bool,
}

impl Default for RideParams {
    fn default() -> Self {
        Self {
            pos: Vec3::new(RIDE_X, RIDE_Y, RIDE_Z),
            restitution: RIDE_RESTITUTION,
            friction: RIDE_FRICTION,
            dirty: false,
        }
    }
}
