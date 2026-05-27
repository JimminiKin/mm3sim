use bevy::prelude::*;

/// Marker on every hittable instrument surface.
/// `channel` matches the programming wheel channel number exactly
/// (e.g., `WHEEL_CH_DROP` = 1 for the snare drum, `WHEEL_CH_VIB_FIRST + bar_idx`
/// for vibraphone bars).
#[derive(Component, Clone, Copy)]
pub struct Instrument {
    pub channel: usize,
}

/// Marks the world-space marble spawn origin for `channel`.
///
/// The marble spawn system reads this entity's `Transform.translation` each
/// frame to determine where a marble should appear. Because the spawn position
/// is stored as a plain `Transform`, moving this entity — or letting
/// `sync_instrument_spawners` track its corresponding `Instrument` — is all
/// that is needed to relocate a channel's drop point.
///
/// One `MarbleSpawner` exists per programming-wheel channel:
/// - Ghost-snare (ch 0): sits at the chute entry, synced from `ChuteParams`.
/// - Snare variants (ch 1–7): sit above the snare drum at their X offset,
///   synced from the snare `Instrument` world position.
/// - Vibraphone bars (ch 8–44): sit above each bar, synced from the bar
///   `Instrument` world position.  Tagged `VibraphoneEntity` so they are
///   automatically recreated when the vibraphone is rebuilt.
///
/// To add a new instrument: spawn an `Instrument` entity + a `MarbleSpawner`
/// entity, add the channel to `CHANNEL_DEFS`, and extend the
/// `sync_instrument_spawners` match arm.
#[derive(Component, Clone, Copy)]
pub struct MarbleSpawner {
    pub channel: usize,
}
