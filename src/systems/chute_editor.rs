use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::chute::{spawn_chute, ChuteSegment};
use crate::components::snare::PivotArm;
use crate::components::vibraphone::{spawn_vibraphone, VibraphoneEntity};
use crate::resources::chute_params::{ChuteParams, DragAxis};
use crate::resources::marble_collisions::MarbleCollisions;
use crate::resources::vibraphone_params::VibraphoneParams;
use crate::systems::marble::AutoSpawn;
use crate::systems::sound::SnareVolume;

#[derive(Resource, Default)]
pub struct SnareFixed(pub bool);

pub fn apply_snare_fixed_system(
    mut commands: Commands,
    snare_fixed: Res<SnareFixed>,
    mut arm: Query<(Entity, &mut LinearVelocity, &mut AngularVelocity), With<PivotArm>>,
) {
    if !snare_fixed.is_changed() {
        return;
    }
    let Ok((entity, mut lin_vel, mut ang_vel)) = arm.single_mut() else {
        return;
    };
    if snare_fixed.0 {
        commands.entity(entity).insert(RigidBody::Static);
        *lin_vel = LinearVelocity::ZERO;
        *ang_vel = AngularVelocity::ZERO;
    } else {
        commands.entity(entity).insert(RigidBody::Dynamic);
    }
}

pub fn chute_editor_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<ChuteParams>,
    mut vib: ResMut<VibraphoneParams>,
    mut marble_col: ResMut<MarbleCollisions>,
    mut snare_fixed: ResMut<SnareFixed>,
    mut snare_volume: ResMut<SnareVolume>,
    mut auto_spawn: ResMut<AutoSpawn>,
) {
    let ctx = contexts.ctx_mut().unwrap();
    egui::Window::new("Parameters")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(egui::RichText::new("Parameters").strong())
                .id_salt("params_header")
                .default_open(true)
                .show(ui, |ui| {
                    let mut changed = false;

                    ui.heading("Options");
                    ui.checkbox(&mut params.handles_visible, "Show handles");

                    let old_col = marble_col.bypass_change_detection().0;
                    let mut new_col = old_col;
                    ui.checkbox(&mut new_col, "Marble-marble collisions");
                    if new_col != old_col {
                        marble_col.0 = new_col;
                    }

                    let old_fixed = snare_fixed.bypass_change_detection().0;
                    let mut new_fixed = old_fixed;
                    ui.checkbox(&mut new_fixed, "Fix snare (freeze arm)");
                    if new_fixed != old_fixed {
                        snare_fixed.0 = new_fixed;
                    }

                    ui.horizontal(|ui| {
                        ui.label("Snare volume:");
                        ui.add(egui::Slider::new(&mut snare_volume.0, 0.0..=1.0).show_value(false));
                        ui.monospace(format!("{:.0}%", snare_volume.0 * 100.0));
                    });

                    ui.separator();
                    ui.heading("Chute position");
                    ui.horizontal(|ui| {
                        ui.label("Drag axis:");
                        ui.radio_value(&mut params.drag_axis, DragAxis::Free, "Free");
                        ui.radio_value(&mut params.drag_axis, DragAxis::Vertical, "Y only");
                        ui.radio_value(&mut params.drag_axis, DragAxis::Horizontal, "Z only");
                    });
                    changed |= point_drag_row(ui, "Exit end", &mut params.exit_pos);

                    ui.separator();
                    ui.heading("Chute shape");
                    changed |= scalar_drag_row(ui, "Exit length (m)", &mut params.exit_length, 0.001, 0.005..=0.50);
                    changed |= angle_drag_row(ui, "Exit angle (°)", &mut params.exit_angle, 0.0..=45.0);
                    changed |= scalar_drag_row(ui, "Curve radius (m)", &mut params.curve_radius, 0.001, 0.005..=1.0);
                    changed |= angle_drag_row(ui, "Slope angle (°)", &mut params.slope_angle, 1.0..=85.0);
                    changed |= scalar_drag_row(ui, "Slope length (m)", &mut params.slope_length, 0.001, 0.01..=1.0);

                    ui.separator();
                    if ui.button("Reset to defaults").clicked() {
                        *params = ChuteParams::default();
                        changed = true;
                    }

                    if changed {
                        params.dirty = true;
                    }

                    ui.separator();
                    ui.heading("Vibraphone");
                    let mut vib_changed = false;
                    vib_changed |= scalar_drag_row(ui, "Row Z (m)", &mut vib.row_z, 0.001, -2.0..=0.0);
                    vib_changed |= scalar_drag_row(ui, "Row Y top (m)", &mut vib.row_y, 0.001, -0.5..=0.5);
                    vib_changed |= scalar_drag_row(ui, "Row X center (m)", &mut vib.row_x_center, 0.001, -1.0..=1.0);
                    vib_changed |= scalar_drag_row(ui, "Bar width (m)", &mut vib.bar_width, 0.0005, 0.010..=0.10);
                    vib_changed |= scalar_drag_row(ui, "Bar spacing (m)", &mut vib.bar_spacing, 0.0005, 0.010..=0.20);
                    vib_changed |= scalar_drag_row(ui, "Bar thickness (m)", &mut vib.bar_thickness, 0.0005, 0.003..=0.05);
                    vib_changed |= scalar_drag_row(ui, "Bar len max (m)", &mut vib.bar_length_max, 0.001, 0.05..=0.80);
                    vib_changed |= scalar_drag_row(ui, "Bar len min (m)", &mut vib.bar_length_min, 0.001, 0.05..=0.50);
                    vib_changed |= scalar_drag_row(ui, "Bar density (kg/m³)", &mut vib.bar_density, 1.0, 500.0..=8000.0);
                    vib_changed |= scalar_drag_row(ui, "Ang. damping", &mut vib.angular_damping, 0.01, 0.0..=20.0);
                    vib_changed |= scalar_drag_row(ui, "Restitution", &mut vib.restitution, 0.01, 0.0..=1.0);
                    vib_changed |= scalar_drag_row(ui, "Friction", &mut vib.friction, 0.01, 0.0..=1.0);
                    vib_changed |= scalar_drag_row(ui, "Arm scale (×bar len)", &mut vib.arm_scale, 0.01, 0.5..=4.0);
                    vib_changed |= scalar_drag_row(ui, "Pivot frac (×bar len)", &mut vib.pivot_frac, 0.005, 0.05..=0.48);
                    vib_changed |= angle_drag_row(ui, "Rest angle (°)", &mut vib.rest_deg, 1.0..=45.0);
                    vib_changed |= angle_drag_row(ui, "Max tilt (°)", &mut vib.max_tilt_deg, 0.5..=30.0);
                    vib_changed |= scalar_drag_row(ui, "CW ratio", &mut vib.cw_weight_ratio, 0.001, 0.5..=2.0);
                    ui.horizontal(|ui| {
                        ui.label("Drop bar index:");
                        let old = vib.drop_bar_index;
                        ui.add(egui::DragValue::new(&mut vib.drop_bar_index).range(0..=36u32));
                        if vib.drop_bar_index != old { vib_changed = true; }
                    });
                    ui.checkbox(&mut vib.spawn_marble, "Spawn vib. marble");
                    if vib_changed {
                        vib.dirty = true;
                    }
                    if ui.button("Reset vibraphone").clicked() {
                        *vib = VibraphoneParams::default();
                        vib.dirty = true;
                    }

                    ui.separator();
                    ui.heading("Batch Runs");
                    ui.horizontal(|ui| {
                        ui.label("Count:");
                        ui.add(egui::DragValue::new(&mut auto_spawn.batch_size).range(1..=1000u32));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Step exit y (mm):");
                        ui.add(egui::DragValue::new(&mut auto_spawn.step_exit_y_mm).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Step slope angle (°):");
                        ui.add(egui::DragValue::new(&mut auto_spawn.step_slope_angle_deg).speed(0.1));
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
                        } else if ui
                            .button(format!("Start {}", auto_spawn.batch_size))
                            .clicked()
                        {
                            auto_spawn.pending = auto_spawn.batch_size;
                            auto_spawn.spawned = 0;
                        }
                    });
                });
        });
}

fn point_drag_row(ui: &mut egui::Ui, label: &str, pt: &mut [f32; 2]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed |= ui
                .add(egui::DragValue::new(&mut pt[1]).prefix("y ").speed(0.001))
                .changed();
            changed |= ui
                .add(egui::DragValue::new(&mut pt[0]).prefix("z ").speed(0.001))
                .changed();
        });
    });
    changed
}

fn scalar_drag_row(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    speed: f64,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = ui
                .add(egui::DragValue::new(val).speed(speed).range(range))
                .changed();
        });
    });
    changed
}

fn angle_drag_row(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = ui
                .add(egui::DragValue::new(val).speed(0.1).range(range))
                .changed();
        });
    });
    changed
}

pub fn rebuild_vibraphone_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<VibraphoneParams>,
    entities: Query<Entity, With<VibraphoneEntity>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    spawn_vibraphone(&mut commands, &mut meshes, &mut materials, &params);
}

pub fn rebuild_chute_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut params: ResMut<ChuteParams>,
    segments: Query<Entity, With<ChuteSegment>>,
) {
    if !params.dirty {
        return;
    }
    params.dirty = false;
    for entity in &segments {
        commands.entity(entity).despawn();
    }
    spawn_chute(&mut commands, &mut meshes, &mut materials, &params);
}
