use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::chute::{spawn_chute, ChuteSegment};
use crate::resources::chute_params::ChuteParams;
use crate::resources::marble_collisions::MarbleCollisions;

pub fn chute_editor_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<ChuteParams>,
    mut marble_col: ResMut<MarbleCollisions>,
) {
    let ctx = contexts.ctx_mut();
    egui::Window::new("Parameters")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Parameters").strong())
                .id_source("params_header")
                .default_open(true)
                .show(ui, |ui| {
                    let mut changed = false;

                    ui.heading("Options");
                    if ui.checkbox(&mut params.straight, "Force straight line chute").changed() {
                        changed = true;
                    }
                    ui.checkbox(&mut params.handles_visible, "Show curve handles");
                    ui.checkbox(&mut params.endpoints_visible, "Show endpoint handles");

                    let old_col = marble_col.bypass_change_detection().0;
                    let mut new_col = old_col;
                    ui.checkbox(&mut new_col, "Marble-marble collisions");
                    if new_col != old_col { marble_col.0 = new_col; }

                    ui.separator();
                    ui.heading("Extremities");
                    drag_row(ui, "End", &mut params.p3, &mut changed);
                    drag_row(ui, "Start", &mut params.p0, &mut changed);

                    ui.separator();
                    ui.heading("Curve Handles");
                    ui.add_enabled_ui(!params.straight, |ui| {
                        drag_row(ui, "CP2 handle 2", &mut params.cp2, &mut changed);
                        drag_row(ui, "CP1 handle 1", &mut params.cp1, &mut changed);
                    });

                    ui.separator();
                    if ui.button("Reset to defaults").clicked() {
                        *params = ChuteParams::default();
                        changed = true;
                    }

                    if changed { params.dirty = true; }
                });
        });
}

fn drag_row(ui: &mut egui::Ui, label: &str, pt: &mut [f32; 2], changed: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            *changed |= ui.add(egui::DragValue::new(&mut pt[1]).prefix("y ").speed(0.05)).changed();
            *changed |= ui.add(egui::DragValue::new(&mut pt[0]).prefix("z ").speed(0.05)).changed();
        });
    });
}

pub fn rebuild_chute_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<ChuteParams>,
    segments: Query<Entity, With<ChuteSegment>>,
) {
    if !params.dirty { return; }
    params.dirty = false;
    for entity in &segments { commands.entity(entity).despawn(); }
    spawn_chute(&mut commands, &mut meshes, &mut materials, &params);
}
