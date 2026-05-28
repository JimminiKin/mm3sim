use bevy::prelude::*;

use crate::resources::constants::*;

#[derive(Resource)]
pub struct KickParams {
    pub pos: Vec3,
    pub restitution: f32,
    pub friction: f32,
    pub dirty: bool,
}

impl Default for KickParams {
    fn default() -> Self {
        Self {
            pos: Vec3::new(KICK_X, KICK_Y, KICK_Z),
            restitution: KICK_RESTITUTION,
            friction: KICK_FRICTION,
            dirty: false,
        }
    }
}
