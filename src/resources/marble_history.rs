#[derive(Clone, Copy)]
pub struct HistorySample {
    pub t: f32,
    pub vy: f32,
    pub vz: f32,
    pub speed: f32,
    pub spin: f32, // surface speed: angvel.length() * MARBLE_RADIUS, same units as velocity
}

#[derive(bevy::prelude::Resource, Default)]
pub struct ChuteMarbleHistory {
    pub samples: Vec<HistorySample>,
}
