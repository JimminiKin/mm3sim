use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::resources::constants::MARBLE_RADIUS;
use crate::resources::marble_history::{ChuteMarbleHistory, HistorySample};
use crate::systems::marble::{ChuteMarble, Marble, SpawnTime};

/// Records one sample per frame from the most recently spawned ChuteMarble.
/// Clears history whenever a new ChuteMarble is added.
pub fn record_chute_marble_system(
    mut history: ResMut<ChuteMarbleHistory>,
    time: Res<Time>,
    added: Query<(), Added<ChuteMarble>>,
    marbles: Query<(&Velocity, &SpawnTime), (With<Marble>, With<ChuteMarble>)>,
) {
    if !added.is_empty() {
        history.samples.clear();
    }

    // Pick the most recently spawned chute marble still in the world.
    let Some((vel, spawn_time)) = marbles
        .iter()
        .max_by(|a, b| a.1.0.partial_cmp(&b.1.0).unwrap_or(std::cmp::Ordering::Equal))
    else {
        return;
    };

    history.samples.push(HistorySample {
        t: time.elapsed_seconds() - spawn_time.0,
        vy: vel.linvel.y,
        vz: vel.linvel.z,
        speed: vel.linvel.length(),
        spin: vel.angvel.length() * MARBLE_RADIUS,
    });
}

pub fn marble_graph_ui(mut contexts: EguiContexts, history: Res<ChuteMarbleHistory>) {
    let ctx = contexts.ctx_mut();
    egui::Window::new("Chute Marble")
        .default_pos([10.0, 440.0])
        .default_size([360.0, 220.0])
        .show(ctx, |ui| {
            if history.samples.is_empty() {
                ui.label("No data — spawn a marble");
                return;
            }

            let pts = |f: fn(&HistorySample) -> f64| -> PlotPoints {
                PlotPoints::from_iter(history.samples.iter().map(|s| [s.t as f64, f(s)]))
            };

            Plot::new("chute_vel")
                .legend(Legend::default())
                .height(ui.available_height())
                .x_axis_label("t (s)")
                .y_axis_label("m/s")
                .show(ui, |p| {
                    p.line(Line::new(pts(|s| s.vy as f64)).name("vy"));
                    p.line(Line::new(pts(|s| s.vz as f64)).name("vz"));
                    p.line(Line::new(pts(|s| s.speed as f64)).name("speed"));
                    p.line(Line::new(pts(|s| s.spin as f64)).name("spin"));
                });
        });
}
