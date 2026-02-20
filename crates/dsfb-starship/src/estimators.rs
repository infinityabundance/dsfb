use nalgebra::{SMatrix, SVector, UnitQuaternion, Vector3};

use dsfb::{DsfbObserver, DsfbParams, DsfbState};

use crate::config::SimConfig;
use crate::physics::{gravity_mps2, TruthState};
use crate::sensors::ImuMeasurement;

#[derive(Debug, Clone)]
pub struct NavState {
    pub pos_n_m: Vector3<f64>,
    pub vel_n_mps: Vector3<f64>,
    pub q_bn: UnitQuaternion<f64>,
    pub omega_b_rps: Vector3<f64>,
}

impl NavState {
    pub fn from_truth_with_seed_error(truth: &TruthState, seed_scale: f64) -> Self {
        let pos_err = Vector3::new(45.0 * seed_scale, -30.0 * seed_scale, 80.0 * seed_scale);
        let vel_err = Vector3::new(-2.5 * seed_scale, 1.8 * seed_scale, -1.2 * seed_scale);
        let att_err = UnitQuaternion::from_euler_angles(
            0.3_f64.to_radians() * seed_scale,
            -0.5_f64.to_radians() * seed_scale,
            0.2_f64.to_radians() * seed_scale,
        );

        Self {
            pos_n_m: truth.pos_n_m + pos_err,
            vel_n_mps: truth.vel_n_mps + vel_err,
            q_bn: truth.q_bn * att_err,
            omega_b_rps: truth.omega_b_rps,
        }
    }

    pub fn propagate(&mut self, specific_force_b_mps2: Vector3<f64>, gyro_b_rps: Vector3<f64>, dt_s: f64) {
        let gyro_b_rps = Vector3::new(
            gyro_b_rps.x.clamp(-0.8, 0.8),
            gyro_b_rps.y.clamp(-0.8, 0.8),
            gyro_b_rps.z.clamp(-0.8, 0.8),
        );
        let specific_force_b_mps2 = Vector3::new(
            specific_force_b_mps2.x.clamp(-60.0, 60.0),
            specific_force_b_mps2.y.clamp(-60.0, 60.0),
            specific_force_b_mps2.z.clamp(-60.0, 60.0),
        );

        let dq = UnitQuaternion::from_scaled_axis(gyro_b_rps * dt_s);
        self.q_bn *= dq;

        let g = gravity_mps2(self.pos_n_m.z.max(0.0));
        let gravity_n = Vector3::new(0.0, 0.0, -g);
        let acc_n = self.q_bn.transform_vector(&specific_force_b_mps2) + gravity_n;

        self.vel_n_mps += acc_n * dt_s;
        let speed = self.vel_n_mps.norm();
        if speed > 7_800.0 {
            self.vel_n_mps *= 7_800.0 / speed;
        }
        self.pos_n_m += self.vel_n_mps * dt_s;
        self.pos_n_m.z = self.pos_n_m.z.max(0.0);

        self.omega_b_rps = gyro_b_rps;
    }

    pub fn position_error_m(&self, truth: &TruthState) -> f64 {
        (self.pos_n_m - truth.pos_n_m).norm()
    }

    pub fn velocity_error_mps(&self, truth: &TruthState) -> f64 {
        (self.vel_n_mps - truth.vel_n_mps).norm()
    }

    pub fn attitude_error_deg(&self, truth: &TruthState) -> f64 {
        let dq = self.q_bn.inverse() * truth.q_bn;
        dq.angle().to_degrees().abs()
    }
}

type Mat6 = SMatrix<f64, 6, 6>;
type Vec6 = SVector<f64, 6>;

pub struct SimpleEkf {
    pub nav: NavState,
    p: Mat6,
    q_diag: Vec6,
    r_diag: Vec6,
}

impl SimpleEkf {
    pub fn new(initial: NavState) -> Self {
        Self {
            nav: initial,
            p: Mat6::identity() * 35.0,
            q_diag: Vec6::new(0.04, 0.04, 0.04, 0.55, 0.55, 0.55),
            r_diag: Vec6::new(25.0, 25.0, 36.0, 4.0, 4.0, 5.0),
        }
    }

    pub fn propagate(&mut self, specific_force_b_mps2: Vector3<f64>, gyro_b_rps: Vector3<f64>, dt_s: f64) {
        self.nav.propagate(specific_force_b_mps2, gyro_b_rps, dt_s);

        let mut a = Mat6::identity();
        a[(0, 3)] = dt_s;
        a[(1, 4)] = dt_s;
        a[(2, 5)] = dt_s;

        let mut q = Mat6::zeros();
        for i in 0..6 {
            q[(i, i)] = self.q_diag[i] * dt_s;
        }

        self.p = a * self.p * a.transpose() + q;
    }

    pub fn update_gnss(&mut self, pos_meas: Vector3<f64>, vel_meas: Vector3<f64>) {
        let x = Vec6::new(
            self.nav.pos_n_m.x,
            self.nav.pos_n_m.y,
            self.nav.pos_n_m.z,
            self.nav.vel_n_mps.x,
            self.nav.vel_n_mps.y,
            self.nav.vel_n_mps.z,
        );
        let z = Vec6::new(
            pos_meas.x, pos_meas.y, pos_meas.z, vel_meas.x, vel_meas.y, vel_meas.z,
        );

        let h = Mat6::identity();
        let mut r = Mat6::zeros();
        for i in 0..6 {
            r[(i, i)] = self.r_diag[i];
        }

        let y = z - h * x;
        let s = h * self.p * h.transpose() + r;

        if let Some(s_inv) = s.try_inverse() {
            let k = self.p * h.transpose() * s_inv;
            let x_upd = x + k * y;

            self.nav.pos_n_m = Vector3::new(x_upd[0], x_upd[1], x_upd[2]);
            self.nav.vel_n_mps = Vector3::new(x_upd[3], x_upd[4], x_upd[5]);

            let i = Mat6::identity();
            self.p = (i - k * h) * self.p;
        }
    }
}

struct AxisFusion {
    observer: DsfbObserver,
    prev_samples: Vec<f64>,
    slew_threshold: f64,
    penalty_gain: f64,
    initialized: bool,
    last_increments: Vec<f64>,
}

impl AxisFusion {
    fn new(params: DsfbParams, channels: usize, slew_threshold: f64, penalty_gain: f64) -> Self {
        Self {
            observer: DsfbObserver::new(params, channels),
            prev_samples: vec![0.0; channels],
            slew_threshold,
            penalty_gain,
            initialized: false,
            last_increments: vec![0.0; channels],
        }
    }

    fn step(&mut self, measurements: &[f64], dt_s: f64) -> f64 {
        if !self.initialized {
            let mean = measurements.iter().copied().sum::<f64>() / measurements.len() as f64;
            self.observer.init(DsfbState::new(mean, 0.0, 0.0));
            self.prev_samples.copy_from_slice(measurements);
            self.initialized = true;
        }

        let pred = self.observer.state().phi + self.observer.state().omega * dt_s;
        let mut adjusted = Vec::with_capacity(measurements.len());

        for (idx, sample) in measurements.iter().enumerate() {
            let sample = if sample.is_finite() { *sample } else { pred };
            let prev = if self.prev_samples[idx].is_finite() {
                self.prev_samples[idx]
            } else {
                sample
            };
            let inc = ((sample - prev) / dt_s).abs().min(1_000.0);
            self.last_increments[idx] = inc;

            let mut y = sample;
            if inc > self.slew_threshold {
                let penalty = ((inc - self.slew_threshold) * self.penalty_gain * dt_s).min(25.0);
                let delta = sample - pred;
                let sign = if delta.abs() < 1.0e-12 {
                    1.0
                } else {
                    delta.signum()
                };
                y += sign * penalty;
            }

            adjusted.push(y);
            self.prev_samples[idx] = sample;
        }

        let fused = self.observer.step(&adjusted, dt_s).phi;
        if fused.is_finite() {
            fused
        } else {
            let mean = adjusted.iter().copied().sum::<f64>() / adjusted.len() as f64;
            self.observer.init(DsfbState::new(mean, 0.0, 0.0));
            mean
        }
    }

    fn weight(&self, channel: usize) -> f64 {
        self.observer.trust_weight(channel)
    }

    fn increment(&self, channel: usize) -> f64 {
        self.last_increments[channel]
    }
}

pub struct DsfbFusionLayer {
    accel_axes: [AxisFusion; 3],
    gyro_axes: [AxisFusion; 3],
    channels: usize,
}

impl DsfbFusionLayer {
    pub fn new(cfg: &SimConfig) -> Self {
        let accel_params = DsfbParams::new(0.82, 0.14, 0.016, cfg.rho, 0.05);
        let gyro_params = DsfbParams::new(0.90, 0.11, 0.012, cfg.rho, 0.003);

        let accel_axes = [
            AxisFusion::new(
                accel_params,
                cfg.imu_count,
                cfg.slew_threshold_accel,
                cfg.slew_penalty_gain,
            ),
            AxisFusion::new(
                accel_params,
                cfg.imu_count,
                cfg.slew_threshold_accel,
                cfg.slew_penalty_gain,
            ),
            AxisFusion::new(
                accel_params,
                cfg.imu_count,
                cfg.slew_threshold_accel,
                cfg.slew_penalty_gain,
            ),
        ];

        let gyro_axes = [
            AxisFusion::new(
                gyro_params,
                cfg.imu_count,
                cfg.slew_threshold_gyro,
                cfg.slew_penalty_gain,
            ),
            AxisFusion::new(
                gyro_params,
                cfg.imu_count,
                cfg.slew_threshold_gyro,
                cfg.slew_penalty_gain,
            ),
            AxisFusion::new(
                gyro_params,
                cfg.imu_count,
                cfg.slew_threshold_gyro,
                cfg.slew_penalty_gain,
            ),
        ];

        Self {
            accel_axes,
            gyro_axes,
            channels: cfg.imu_count,
        }
    }

    pub fn fuse(&mut self, measurements: &[ImuMeasurement], dt_s: f64) -> DsfbFusionOutput {
        let mut acc_samples = [vec![0.0_f64; self.channels], vec![0.0_f64; self.channels], vec![0.0_f64; self.channels]];
        let mut gyr_samples = [vec![0.0_f64; self.channels], vec![0.0_f64; self.channels], vec![0.0_f64; self.channels]];

        for (idx, m) in measurements.iter().enumerate() {
            acc_samples[0][idx] = m.accel_b_mps2.x;
            acc_samples[1][idx] = m.accel_b_mps2.y;
            acc_samples[2][idx] = m.accel_b_mps2.z;

            gyr_samples[0][idx] = m.gyro_b_rps.x;
            gyr_samples[1][idx] = m.gyro_b_rps.y;
            gyr_samples[2][idx] = m.gyro_b_rps.z;
        }

        let fused_accel = Vector3::new(
            self.accel_axes[0].step(&acc_samples[0], dt_s),
            self.accel_axes[1].step(&acc_samples[1], dt_s),
            self.accel_axes[2].step(&acc_samples[2], dt_s),
        );

        let fused_gyro = Vector3::new(
            self.gyro_axes[0].step(&gyr_samples[0], dt_s),
            self.gyro_axes[1].step(&gyr_samples[1], dt_s),
            self.gyro_axes[2].step(&gyr_samples[2], dt_s),
        );

        let mut trust_weights = vec![0.0; self.channels];
        let mut residual_increments = vec![0.0; self.channels];

        for ch in 0..self.channels {
            let mut w_sum = 0.0;
            let mut inc_sum = 0.0;

            for axis in &self.accel_axes {
                w_sum += axis.weight(ch);
                inc_sum += axis.increment(ch);
            }
            for axis in &self.gyro_axes {
                w_sum += axis.weight(ch);
                inc_sum += axis.increment(ch);
            }

            trust_weights[ch] = w_sum / 6.0;
            residual_increments[ch] = inc_sum / 6.0;
        }

        DsfbFusionOutput {
            fused_accel_b_mps2: fused_accel,
            fused_gyro_b_rps: fused_gyro,
            trust_weights,
            residual_increments,
        }
    }
}

pub struct DsfbFusionOutput {
    pub fused_accel_b_mps2: Vector3<f64>,
    pub fused_gyro_b_rps: Vector3<f64>,
    pub trust_weights: Vec<f64>,
    pub residual_increments: Vec<f64>,
}

pub fn mean_measurement(measurements: &[ImuMeasurement]) -> ImuMeasurement {
    let n = measurements.len() as f64;

    let mut acc = Vector3::zeros();
    let mut gyro = Vector3::zeros();
    for m in measurements {
        acc += m.accel_b_mps2;
        gyro += m.gyro_b_rps;
    }

    ImuMeasurement {
        accel_b_mps2: acc / n,
        gyro_b_rps: gyro / n,
    }
}
