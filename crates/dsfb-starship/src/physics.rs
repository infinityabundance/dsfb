use std::f64::consts::PI;

use nalgebra::{Matrix3, UnitQuaternion, Vector3};

use crate::config::SimConfig;

const EARTH_RADIUS_M: f64 = 6_371_000.0;
const G0: f64 = 9.80665;
const R_AIR: f64 = 287.05;
const GAMMA_AIR: f64 = 1.4;
const SIGMA_SB: f64 = 5.670_374_419e-8;

#[derive(Debug, Clone)]
pub struct VehicleParams {
    pub dry_mass_kg: f64,
    pub entry_mass_kg: f64,
    pub ref_area_m2: f64,
    pub ref_span_m: f64,
    pub ref_length_m: f64,
    pub nose_radius_m: f64,
    pub inertia_kgm2: Matrix3<f64>,
    pub inertia_inv_kgm2: Matrix3<f64>,
}

impl Default for VehicleParams {
    fn default() -> Self {
        let inertia_kgm2 = Matrix3::new(
            1.9e7, 0.0, 0.0, // Ixx
            0.0, 1.5e7, 0.0, // Iyy
            0.0, 0.0, 2.1e7, // Izz
        );
        let inertia_inv_kgm2 = inertia_kgm2
            .try_inverse()
            .expect("inertia matrix must be invertible");

        Self {
            dry_mass_kg: 95_000.0,
            entry_mass_kg: 120_000.0,
            ref_area_m2: 340.0,
            ref_span_m: 9.0,
            ref_length_m: 50.0,
            nose_radius_m: 1.8,
            inertia_kgm2,
            inertia_inv_kgm2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TruthState {
    pub pos_n_m: Vector3<f64>,
    pub vel_n_mps: Vector3<f64>,
    pub q_bn: UnitQuaternion<f64>,
    pub omega_b_rps: Vector3<f64>,
    pub mass_kg: f64,
    pub heat_shield_temp_k: f64,
}

impl TruthState {
    pub fn altitude_m(&self) -> f64 {
        self.pos_n_m.z.max(0.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AtmosphereSample {
    pub density_kg_m3: f64,
    pub pressure_pa: f64,
    pub temperature_k: f64,
    pub sound_speed_mps: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AeroSample {
    pub specific_force_b_mps2: Vector3<f64>,
    pub moment_b_nm: Vector3<f64>,
    pub dynamic_pressure_pa: f64,
    pub mach: f64,
    pub alpha_deg: f64,
    pub beta_deg: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct TruthStepSample {
    pub atmosphere: AtmosphereSample,
    pub aero: AeroSample,
    pub angular_accel_b_rps2: Vector3<f64>,
    pub heat_flux_w_m2: f64,
    pub blackout: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ReentryEventState {
    pub tile_loss_active: bool,
}

impl Default for ReentryEventState {
    fn default() -> Self {
        Self {
            tile_loss_active: false,
        }
    }
}

pub fn initial_truth_state(cfg: &SimConfig, params: &VehicleParams) -> TruthState {
    let gamma = cfg.entry_flight_path_deg.to_radians();
    let speed = cfg.entry_speed_mps;
    let vel_n_mps = Vector3::new(speed * gamma.cos(), 0.0, speed * gamma.sin());

    // Body frame initially aligned with trajectory with a slight nose-up offset.
    let q_bn = UnitQuaternion::from_euler_angles(0.0, 22.0_f64.to_radians(), 0.0);

    TruthState {
        pos_n_m: Vector3::new(0.0, 0.0, cfg.entry_altitude_m),
        vel_n_mps,
        q_bn,
        omega_b_rps: Vector3::new(0.0, 0.0, 0.0),
        mass_kg: params.entry_mass_kg,
        heat_shield_temp_k: 320.0,
    }
}

pub fn gravity_mps2(altitude_m: f64) -> f64 {
    G0 * (EARTH_RADIUS_M / (EARTH_RADIUS_M + altitude_m.max(0.0))).powi(2)
}

pub fn atmosphere_sample(altitude_m: f64) -> AtmosphereSample {
    let h = altitude_m.max(0.0);
    let rho0 = 1.225;
    let scale_height = 7_200.0;
    let density_kg_m3 = (rho0 * (-h / scale_height).exp()).max(1.0e-7);

    let temperature_k = if h <= 11_000.0 {
        288.15 - 0.0065 * h
    } else if h <= 20_000.0 {
        216.65
    } else if h <= 47_000.0 {
        216.65 + 0.0014 * (h - 20_000.0)
    } else {
        (255.0 - 0.0012 * (h - 47_000.0)).max(165.0)
    };

    let pressure_pa = density_kg_m3 * R_AIR * temperature_k;
    let sound_speed_mps = (GAMMA_AIR * R_AIR * temperature_k).sqrt();

    AtmosphereSample {
        density_kg_m3,
        pressure_pa,
        temperature_k,
        sound_speed_mps,
    }
}

fn target_alpha_rad(altitude_m: f64) -> f64 {
    let alpha_deg = if altitude_m > 95_000.0 {
        24.0
    } else if altitude_m > 75_000.0 {
        24.0 + (95_000.0 - altitude_m) / 20_000.0 * 18.0
    } else if altitude_m > 50_000.0 {
        42.0 + (75_000.0 - altitude_m) / 25_000.0 * 16.0
    } else if altitude_m > 30_000.0 {
        58.0 - (50_000.0 - altitude_m) / 20_000.0 * 10.0
    } else {
        48.0
    };
    alpha_deg.to_radians()
}

fn smooth_pulse(t: f64, start: f64, duration: f64, amplitude: f64) -> f64 {
    if !(start..=start + duration).contains(&t) {
        return 0.0;
    }
    let tau = (t - start) / duration;
    let window = 0.5 - 0.5 * (2.0 * PI * tau).cos();
    amplitude * window
}

fn aerodynamic_sample(
    state: &TruthState,
    params: &VehicleParams,
    atmosphere: AtmosphereSample,
    t_s: f64,
    events: &ReentryEventState,
) -> AeroSample {
    let v_n = state.vel_n_mps;
    let speed = v_n.norm().max(1.0);
    let v_b = state.q_bn.inverse_transform_vector(&v_n);

    let alpha_raw = v_b.z.atan2(v_b.x);
    let beta_raw = (v_b.y / speed).clamp(-0.95, 0.95).asin();
    let alpha = alpha_raw.clamp(-70.0_f64.to_radians(), 70.0_f64.to_radians());
    let beta = beta_raw.clamp(-25.0_f64.to_radians(), 25.0_f64.to_radians());
    let mach = speed / atmosphere.sound_speed_mps.max(1.0);
    let q_dyn_raw = 0.5 * atmosphere.density_kg_m3 * speed * speed;
    let q_dyn = q_dyn_raw.min(85_000.0);

    let target_alpha = target_alpha_rad(state.altitude_m());
    let pitch_err = target_alpha - alpha;
    let pitch_cmd = (1.35 * pitch_err - 0.28 * state.omega_b_rps.y).clamp(-0.70, 0.70);
    let yaw_cmd = (-0.9 * beta - 0.22 * state.omega_b_rps.z).clamp(-0.45, 0.45);
    let bank_cmd = (12.0_f64.to_radians() * (0.0052 * t_s).sin()).clamp(-0.30, 0.30);

    let transient_pitch = smooth_pulse(t_s, 205.0, 9.0, 0.23);
    let transient_roll = smooth_pulse(t_s, 274.0, 12.0, 0.17);
    let transient_yaw = smooth_pulse(t_s, 283.0, 15.0, -0.12);

    let asym_side = if events.tile_loss_active { 0.085 } else { 0.0 };
    let asym_roll = if events.tile_loss_active { 0.065 } else { 0.0 };
    let asym_yaw = if events.tile_loss_active { -0.045 } else { 0.0 };

    let cd = (0.92 + 0.75 * alpha.sin().abs() + 0.02 * (mach - 6.0).max(0.0).min(10.0)).clamp(0.5, 2.4);
    let cl = (1.45 * alpha.sin() + 0.22 * pitch_cmd).clamp(-1.2, 1.9);
    let cy = (-0.50 * beta + 0.10 * yaw_cmd + asym_side + 0.03 * transient_yaw).clamp(-0.7, 0.7);

    let p_hat = state.omega_b_rps.x * params.ref_span_m / (2.0 * speed);
    let q_hat = state.omega_b_rps.y * params.ref_length_m / (2.0 * speed);
    let r_hat = state.omega_b_rps.z * params.ref_span_m / (2.0 * speed);

    let c_roll = (-0.18 * beta - 0.62 * p_hat + 0.22 * bank_cmd + asym_roll + transient_roll).clamp(-0.65, 0.65);
    let c_pitch = (-0.48 * (alpha - target_alpha) - 0.58 * q_hat + 0.48 * pitch_cmd + transient_pitch)
        .clamp(-0.75, 0.75);
    let c_yaw = (-0.24 * beta - 0.54 * r_hat + 0.42 * yaw_cmd + asym_yaw + transient_yaw).clamp(-0.65, 0.65);

    let force_b = q_dyn
        * params.ref_area_m2
        * Vector3::new(
            -cd,
            cy,
            cl,
        );
    let mut moment_b = Vector3::new(
        q_dyn * params.ref_area_m2 * params.ref_span_m * c_roll,
        q_dyn * params.ref_area_m2 * params.ref_length_m * c_pitch,
        q_dyn * params.ref_area_m2 * params.ref_span_m * c_yaw,
    );
    moment_b.x = moment_b.x.clamp(-4.0e6, 4.0e6);
    moment_b.y = moment_b.y.clamp(-5.5e6, 5.5e6);
    moment_b.z = moment_b.z.clamp(-4.0e6, 4.0e6);

    let specific_force_b_mps2 = force_b / state.mass_kg.max(params.dry_mass_kg);

    AeroSample {
        specific_force_b_mps2,
        moment_b_nm: moment_b,
        dynamic_pressure_pa: q_dyn_raw,
        mach,
        alpha_deg: alpha.to_degrees(),
        beta_deg: beta.to_degrees(),
    }
}

pub fn truth_step(
    state: &mut TruthState,
    params: &VehicleParams,
    cfg: &SimConfig,
    t_s: f64,
    dt_s: f64,
    events: &mut ReentryEventState,
) -> TruthStepSample {
    if t_s >= 320.0 {
        events.tile_loss_active = true;
    }

    let atmosphere = atmosphere_sample(state.altitude_m());
    let aero = aerodynamic_sample(state, params, atmosphere, t_s, events);

    let g = gravity_mps2(state.altitude_m());
    let gravity_n = Vector3::new(0.0, 0.0, -g);
    let acc_n = state.q_bn.transform_vector(&aero.specific_force_b_mps2) + gravity_n;

    state.vel_n_mps += acc_n * dt_s;

    // Guidance shaping: sustain a shallow descent during plasma blackout altitudes.
    if (cfg.blackout_lower_m..=cfg.blackout_upper_m).contains(&state.altitude_m()) {
        let target_vz = -110.0 - 15.0 * (0.0025 * t_s).sin();
        state.vel_n_mps.z = 0.75 * state.vel_n_mps.z + 0.25 * target_vz;
    }

    let speed = state.vel_n_mps.norm();
    if speed > 7_700.0 {
        state.vel_n_mps *= 7_700.0 / speed;
    }

    state.pos_n_m += state.vel_n_mps * dt_s;
    state.pos_n_m.z = state.pos_n_m.z.max(0.0);

    let coriolis = state
        .omega_b_rps
        .cross(&(params.inertia_kgm2 * state.omega_b_rps));
    let omega_dot = params.inertia_inv_kgm2 * (aero.moment_b_nm - coriolis);
    state.omega_b_rps += omega_dot * dt_s;
    state.omega_b_rps.x = state.omega_b_rps.x.clamp(-0.45, 0.45);
    state.omega_b_rps.y = state.omega_b_rps.y.clamp(-0.50, 0.50);
    state.omega_b_rps.z = state.omega_b_rps.z.clamp(-0.45, 0.45);

    let dq = UnitQuaternion::from_scaled_axis(state.omega_b_rps * dt_s);
    state.q_bn *= dq;

    // Sutton-Graves-like convective stagnation heating estimate.
    let speed = state.vel_n_mps.norm();
    let heat_flux = 1.1e-4
        * (atmosphere.density_kg_m3 / params.nose_radius_m)
            .sqrt()
            .max(0.0)
        * speed.powi(3);

    let ambient_k = atmosphere.temperature_k;
    let q_rad = 0.82 * SIGMA_SB * (state.heat_shield_temp_k.powi(4) - ambient_k.powi(4)).max(0.0);
    let thermal_capacity = 7.5e5;
    let temp_dot = (0.095 * heat_flux - q_rad) / thermal_capacity;
    state.heat_shield_temp_k = (state.heat_shield_temp_k + temp_dot * dt_s).clamp(280.0, 2_100.0);

    let mass_dot = -1.1e-7 * heat_flux * params.ref_area_m2;
    state.mass_kg = (state.mass_kg + mass_dot * dt_s).max(params.dry_mass_kg);

    let blackout = state.altitude_m() <= cfg.blackout_upper_m && state.altitude_m() >= cfg.blackout_lower_m;

    TruthStepSample {
        atmosphere,
        aero,
        angular_accel_b_rps2: omega_dot,
        heat_flux_w_m2: heat_flux,
        blackout,
    }
}
