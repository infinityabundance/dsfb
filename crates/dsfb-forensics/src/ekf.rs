//! Simple EKF baseline used as the stochastic shadow.
//!
//! References: `CORE-08` for anomaly acceptance semantics and `DSFB-07` for
//! residual consistency against the forward image. The model is linear in this
//! crate, but is implemented in EKF form so the baseline stays structurally
//! aligned with the wider DSFB stack language.

use dsfb::DsfbState;
/// Measurement decision for one EKF update.
#[derive(Clone, Copy, Debug)]
pub struct EkfMeasurementDecision {
    /// Channel index.
    pub channel_index: usize,
    /// Measurement innovation.
    pub innovation: f64,
    /// Innovation variance.
    pub innovation_variance: f64,
    /// Normalized innovation squared.
    pub nis: f64,
    /// Whether the EKF accepted the measurement.
    pub accepted: bool,
}

/// EKF step result across all channels.
#[derive(Clone, Debug)]
pub struct EkfStepResult {
    /// Updated state estimate after the step.
    pub state: DsfbState,
    /// One decision per input channel.
    pub decisions: Vec<EkfMeasurementDecision>,
}

/// Minimal EKF baseline observer.
pub struct BaselineEkf {
    state: [f64; 3],
    covariance: [[f64; 3]; 3],
    process_noise: f64,
    measurement_noise: f64,
    acceptance_gate: f64,
}

impl BaselineEkf {
    /// Construct the EKF baseline.
    ///
    /// References: `CORE-08` and `CORE-10`.
    pub fn new(initial_state: DsfbState) -> Self {
        Self {
            state: [initial_state.phi, initial_state.omega, initial_state.alpha],
            covariance: [[0.8, 0.0, 0.0], [0.0, 0.8, 0.0], [0.0, 0.0, 0.8]],
            process_noise: 0.02,
            measurement_noise: 0.35,
            acceptance_gate: 16.0,
        }
    }

    /// Advance the EKF baseline by one step.
    ///
    /// References: `CORE-08`, `DSFB-07`, and `DSFB-08`.
    pub fn step(&mut self, measurements: &[f64], dt: f64) -> EkfStepResult {
        self.predict(dt);
        let mut decisions = Vec::with_capacity(measurements.len());
        for (channel_index, &measurement) in measurements.iter().enumerate() {
            let innovation = measurement - self.state[0];
            let innovation_variance = self.covariance[0][0] + self.measurement_noise;
            let nis = if innovation_variance > 0.0 {
                innovation * innovation / innovation_variance
            } else {
                f64::INFINITY
            };
            let accepted = nis <= self.acceptance_gate;
            if accepted {
                self.update_with_innovation(innovation, innovation_variance);
            }
            decisions.push(EkfMeasurementDecision {
                channel_index,
                innovation,
                innovation_variance,
                nis,
                accepted,
            });
        }

        EkfStepResult {
            state: DsfbState::new(self.state[0], self.state[1], self.state[2]),
            decisions,
        }
    }

    fn predict(&mut self, dt: f64) {
        let dt2 = dt * dt;
        let half_dt2 = 0.5 * dt2;
        let transition = [
            [1.0, dt, half_dt2],
            [0.0, 1.0, dt],
            [0.0, 0.0, 1.0],
        ];
        self.state = [
            self.state[0] + self.state[1] * dt + self.state[2] * half_dt2,
            self.state[1] + self.state[2] * dt,
            self.state[2],
        ];
        let predicted = multiply_3x3(transition, self.covariance);
        let predicted = multiply_3x3_transpose(predicted, transition);
        let q = self.process_covariance(dt);
        self.covariance = add_3x3(predicted, q);
    }

    fn update_with_innovation(&mut self, innovation: f64, innovation_variance: f64) {
        let gain = [
            self.covariance[0][0] / innovation_variance,
            self.covariance[1][0] / innovation_variance,
            self.covariance[2][0] / innovation_variance,
        ];
        for (index, value) in self.state.iter_mut().enumerate() {
            *value += gain[index] * innovation;
        }

        let row0 = self.covariance[0];
        let mut next = self.covariance;
        for row in 0..3 {
            for col in 0..3 {
                next[row][col] -= gain[row] * row0[col];
            }
        }
        self.covariance = next;
    }

    fn process_covariance(&self, dt: f64) -> [[f64; 3]; 3] {
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;
        let dt4 = dt2 * dt2;
        let q = self.process_noise;
        [
            [0.25 * dt4 * q, 0.5 * dt3 * q, 0.5 * dt2 * q],
            [0.5 * dt3 * q, dt2 * q, dt * q],
            [0.5 * dt2 * q, dt * q, q],
        ]
    }
}

fn multiply_3x3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for row in 0..3 {
        for col in 0..3 {
            out[row][col] = a[row][0] * b[0][col] + a[row][1] * b[1][col] + a[row][2] * b[2][col];
        }
    }
    out
}

fn multiply_3x3_transpose(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for row in 0..3 {
        for col in 0..3 {
            out[row][col] = a[row][0] * b[col][0] + a[row][1] * b[col][1] + a[row][2] * b[col][2];
        }
    }
    out
}

fn add_3x3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for row in 0..3 {
        for col in 0..3 {
            out[row][col] = a[row][col] + b[row][col];
        }
    }
    out
}
