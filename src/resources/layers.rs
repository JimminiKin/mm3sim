use avian3d::prelude::PhysicsLayer;

/// Collision layer assignments.
/// Marbles are in the Marble layer; snare, chute, and arm use Default (collide with everything).
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Marble,
}
