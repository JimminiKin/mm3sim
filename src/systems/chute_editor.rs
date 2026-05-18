use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;

use crate::components::chute::{spawn_chute, ChuteSegment};
use crate::components::snare::PivotArm;
use crate::resources::chute_params::{ChuteParams, DragAxis};
use crate::resources::marble_collisions::MarbleCollisions;
use crate::systems::marble::AutoSpawn;
use crate::systems::sound::SnareVolume;

#[derive(Resource, Default)]
pub struct SnareFixed(pub bool);

pub fn apply_snare_fixed_system(
    snare_fixed: Res<SnareFixed>,
    mut arm: Query<(&mut RigidBody, &mut Velocity), With<PivotArm>>,
) {
    if !snare_fixed.is_changed() { return; }
    let Ok((mut rb, mut vel)) = arm.get_single_mut() else { return };
    if snare_fixed.0 {
        *rb = RigidBody::Fixed;
        *vel = Velocity::zero();
    } else {
        *rb = RigidBody::Dynamic;
    }
}

pub fn chute_editor_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<ChuteParams>,
    mut marble_col: ResMut<MarbleCollisions>,
    mut snare_fixed: ResMut<SnareFixed>,
    mut snare_volume: ResMut<SnareVolume>,
    mut auto_spawn: ResMut<AutoSpawn>,
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

                    ui.horizontal(|ui| {
                        ui.label("Drag axis:");
                        ui.radio_value(&mut params.drag_axis, DragAxis::Free,       "Free");
                        ui.radio_value(&mut params.drag_axis, DragAxis::Vertical,   "Y only");
                        ui.radio_value(&mut params.drag_axis, DragAxis::Horizontal, "Z only");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Shift Y:");
                        if ui.button("− 1mm").clicked() {
                            params.p0[1]  -= 0.001;
                            params.cp1[1] -= 0.001;
                            params.cp2[1] -= 0.001;
                            params.p3[1]  -= 0.001;
                            changed = true;
                        }
                        if ui.button("+ 1mm").clicked() {
                            params.p0[1]  += 0.001;
                            params.cp1[1] += 0.001;
                            params.cp2[1] += 0.001;
                            params.p3[1]  += 0.001;
                            changed = true;
                        }
                    });

                    let old_col = marble_col.bypass_change_detection().0;
                    let mut new_col = old_col;
                    ui.checkbox(&mut new_col, "Marble-marble collisions");
                    if new_col != old_col { marble_col.0 = new_col; }

                    let old_fixed = snare_fixed.bypass_change_detection().0;
                    let mut new_fixed = old_fixed;
                    ui.checkbox(&mut new_fixed, "Fix snare (freeze arm)");
                    if new_fixed != old_fixed { snare_fixed.0 = new_fixed; }

                    ui.horizontal(|ui| {
                        ui.label("Snare volume:");
                        ui.add(egui::Slider::new(&mut snare_volume.0, 0.0..=1.0).show_value(false));
                        ui.monospace(format!("{:.0}%", snare_volume.0 * 100.0));
                    });

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

                    ui.separator();
                    ui.heading("Batch Runs");
                    ui.horizontal(|ui| {
                        ui.label("Count:");
                        ui.add(egui::DragValue::new(&mut auto_spawn.batch_size).range(1..=1000u32));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Step P3.y (mm):");
                        ui.add(egui::DragValue::new(&mut auto_spawn.step_y_mm).speed(0.1));
                    });

                    let is_running = auto_spawn.pending > 0 || auto_spawn.waiting_for.is_some();
                    ui.horizontal(|ui| {
                        if is_running {
                            let done = auto_spawn.spawned;
                            let total = done + auto_spawn.pending;
                            ui.label(format!("{done}/{total}"));
                            if ui.button("Stop").clicked() {
                                auto_spawn.pending = 0;
                                auto_spawn.waiting_for = None;
                            }
                        } else {
                            if ui.button(format!("Start {}", auto_spawn.batch_size)).clicked() {
                                auto_spawn.pending = auto_spawn.batch_size;
                                auto_spawn.spawned = 0;
                            }
                        }
                    });
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
