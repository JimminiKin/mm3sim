use bevy::prelude::*;

#[derive(Resource)]
pub struct PowerParams {
    pub machine_height_m: f32,
    pub spring_k_n_per_m: f32,
    pub spring_compression_cm: f32,
    pub actuator_mass_g: f32,
    pub actuator_travel_cm: f32,
    pub actuator_time_ms: f32,
    pub hihat_close_mass_g: f32,
    pub hihat_close_travel_cm: f32,
    pub carousel_inertia_kg_m2: f32,
    pub carousel_omega_rad_s: f32,
    pub wheel_mass_kg: f32,
    pub bearing_friction_mu: f32,
    pub bearing_radius_mm: f32,
    pub show_graph: bool,
}

impl Default for PowerParams {
    fn default() -> Self {
        Self {
            machine_height_m: 2.0,
            spring_k_n_per_m: 50.0,
            spring_compression_cm: 1.0,
            actuator_mass_g: 30.0,
            actuator_travel_cm: 1.0,
            actuator_time_ms: 50.0,
            hihat_close_mass_g: 50.0,
            // HIHAT_GAP_OPEN (2.5 cm) − HIHAT_GAP_CLOSED (0.4 cm)
            hihat_close_travel_cm: 2.1,
            carousel_inertia_kg_m2: 0.010,
            carousel_omega_rad_s: 5.0,
            wheel_mass_kg: 2.0,
            bearing_friction_mu: 0.001,
            bearing_radius_mm: 10.0,
            show_graph: true,
        }
    }
}
