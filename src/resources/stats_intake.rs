use bevy::prelude::*;

/// When `false`, per-frame stats collectors (`record_marble_samples_system` and
/// `record_marble_paths_system`) are skipped entirely via Bevy run-conditions.
/// Hit records and sound are unaffected.  Toggle from the Chute Editor panel.
#[derive(Resource)]
pub struct StatsIntake(pub bool);

impl Default for StatsIntake {
    fn default() -> Self {
        Self(true)
    }
}
