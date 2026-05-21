use bevy::prelude::*;

/// Marker on every hittable instrument surface.
/// `channel` matches the programming wheel channel: 1 = snare, 2..=38 = vib bars 0..36.
/// Adding a new instrument: spawn its collider with this component at the right channel.
#[derive(Component, Clone, Copy)]
pub struct Instrument {
    pub channel: usize,
}
