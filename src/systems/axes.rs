use bevy::prelude::*;

use crate::resources::constants::AXIS_LENGTH;

pub fn draw_axes_system(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::X * AXIS_LENGTH, Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y * AXIS_LENGTH, Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z * AXIS_LENGTH, Color::BLUE);
}
