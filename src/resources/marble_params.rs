use bevy::prelude::*;
use crate::resources::constants::MARBLE_RADIUS;

const STEEL_DENSITY: f32 = 7850.0;

pub fn mass_for_radius(radius: f32) -> f32 {
    STEEL_DENSITY * (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3)
}

#[derive(Resource)]
pub struct MarbleParams {
    pub radius: f32,
    pub mass: f32,
}

impl MarbleParams {
    pub fn from_radius(radius: f32) -> Self {
        Self { radius, mass: mass_for_radius(radius) }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.mass = mass_for_radius(radius);
    }
}

impl Default for MarbleParams {
    fn default() -> Self {
        Self::from_radius(MARBLE_RADIUS)
    }
}
