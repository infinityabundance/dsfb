use std::f64::consts::PI;

use nalgebra::Vector3;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::StandardNormal;

use crate::physics::ReentryEventState;

#[derive(Debug, Clone, Copy)]
pub struct ImuMeasurement {
    pub accel_b_mps2: Vector3<f64>,
    pub gyro_b_rps: Vector3<f64>,
}

#[derive(Debug, Clone)]
struct ImuChannel {
    accel_bias0: Vector3<f64>,
    gyro_bias0: Vector3<f64>,
    accel_drift_rate: Vector3<f64>,
    gyro_drift_rate: Vector3<f64>,
    accel_noise_std: f64,
    gyro_noise_std: f64,
    accel_thermal_coeff: Vector3<f64>,
    gyro_thermal_coeff: Vector3<f64>,
}

pub struct ImuArray {
    channels: Vec<ImuChannel>,
    rng: ChaCha8Rng,
}

impl ImuArray {
    pub fn new(seed: u64, count: usize) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xBAD5EED_u64);
        let mut channels = Vec::with_capacity(count);

        for idx in 0..count {
            let channel_scale = 1.0 + 0.11 * idx as f64;
            let accel_bias0 = Vector3::new(
                0.03 * channel_scale,
                -0.02 * channel_scale,
                0.05 * channel_scale,
            );
            let gyro_bias0 = Vector3::new(
                0.0009 * channel_scale,
                -0.0011 * channel_scale,
                0.0007 * channel_scale,
            );

            let accel_drift_rate = Vector3::new(
                1.8e-4 * (1.0 + rng.gen::<f64>() * 0.2),
                -1.2e-4 * (1.0 + rng.gen::<f64>() * 0.2),
                2.1e-4 * (1.0 + rng.gen::<f64>() * 0.2),
            );
            let gyro_drift_rate = Vector3::new(
                1.2e-5 * (1.0 + rng.gen::<f64>() * 0.3),
                -1.6e-5 * (1.0 + rng.gen::<f64>() * 0.3),
                1.0e-5 * (1.0 + rng.gen::<f64>() * 0.3),
            );

            channels.push(ImuChannel {
                accel_bias0,
                gyro_bias0,
                accel_drift_rate,
                gyro_drift_rate,
                accel_noise_std: 0.045 + 0.01 * idx as f64,
                gyro_noise_std: 0.0012 + 0.0003 * idx as f64,
                accel_thermal_coeff: Vector3::new(4.0e-4, -2.5e-4, 6.0e-4),
                gyro_thermal_coeff: Vector3::new(4.0e-6, -2.2e-6, 3.0e-6),
            });
        }

        Self { channels, rng }
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    pub fn measure(
        &mut self,
        true_specific_force_b_mps2: Vector3<f64>,
        true_gyro_b_rps: Vector3<f64>,
        heat_shield_temp_k: f64,
        t_s: f64,
        events: &ReentryEventState,
    ) -> Vec<ImuMeasurement> {
        let mut out = Vec::with_capacity(self.channels.len());

        for idx in 0..self.channels.len() {
            let channel = self.channels[idx].clone();
            let thermal_delta = (heat_shield_temp_k - 320.0).max(0.0);

            let accel_bias = channel.accel_bias0
                + channel.accel_drift_rate * t_s
                + channel.accel_thermal_coeff * thermal_delta;
            let gyro_bias = channel.gyro_bias0
                + channel.gyro_drift_rate * t_s
                + channel.gyro_thermal_coeff * thermal_delta;

            let accel_noise = Vector3::new(
                self.gaussian(channel.accel_noise_std),
                self.gaussian(channel.accel_noise_std),
                self.gaussian(channel.accel_noise_std),
            );
            let gyro_noise = Vector3::new(
                self.gaussian(channel.gyro_noise_std),
                self.gaussian(channel.gyro_noise_std),
                self.gaussian(channel.gyro_noise_std),
            );

            let (accel_fault, gyro_fault) = fault_terms(idx, t_s, events);

            out.push(ImuMeasurement {
                accel_b_mps2: true_specific_force_b_mps2 + accel_bias + accel_noise + accel_fault,
                gyro_b_rps: true_gyro_b_rps + gyro_bias + gyro_noise + gyro_fault,
            });
        }

        out
    }

    fn gaussian(&mut self, sigma: f64) -> f64 {
        let z: f64 = self.rng.sample(StandardNormal);
        sigma * z
    }
}

fn smooth_pulse(t: f64, start: f64, duration: f64, amplitude: f64) -> f64 {
    if !(start..=start + duration).contains(&t) {
        return 0.0;
    }
    let tau = (t - start) / duration;
    amplitude * (0.5 - 0.5 * (2.0 * PI * tau).cos())
}

fn fault_terms(idx: usize, t_s: f64, events: &ReentryEventState) -> (Vector3<f64>, Vector3<f64>) {
    // Channel 1 receives the strongest abrupt slew events.
    let mut accel_fault = Vector3::zeros();
    let mut gyro_fault = Vector3::zeros();

    if idx == 1 {
        accel_fault.z += smooth_pulse(t_s, 205.0, 6.0, 22.0);
        accel_fault.y += smooth_pulse(t_s, 274.0, 10.0, 10.0);
        gyro_fault.y += smooth_pulse(t_s, 274.0, 8.0, 0.90);
        gyro_fault.z += smooth_pulse(t_s, 283.0, 12.0, -0.62);

        if events.tile_loss_active {
            accel_fault += Vector3::new(1.35, 0.85, 2.10);
            gyro_fault += Vector3::new(0.038, -0.044, 0.052);
        }
    }

    // Channel 2 has milder but non-negligible drift-like transients.
    if idx == 2 {
        accel_fault.x += smooth_pulse(t_s, 210.0, 9.0, 1.6);
        gyro_fault.x += smooth_pulse(t_s, 286.0, 11.0, 0.07);

        if events.tile_loss_active {
            accel_fault += Vector3::new(-0.12, 0.14, 0.30);
            gyro_fault += Vector3::new(-0.005, 0.004, -0.006);
        }
    }

    (accel_fault, gyro_fault)
}
