use serde::{Deserialize, Serialize};

/// Runtime configuration for the Starship re-entry DSFB demonstration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    /// Fixed integration step [s]
    pub dt: f64,
    /// Final simulation time [s]
    pub t_final: f64,
    /// Number of redundant IMU channels
    pub imu_count: usize,
    /// RNG seed for reproducibility
    pub seed: u64,
    /// Altitude where blackout starts [m]
    pub blackout_upper_m: f64,
    /// Altitude where blackout ends [m]
    pub blackout_lower_m: f64,
    /// Atmospheric entry interface altitude [m]
    pub entry_altitude_m: f64,
    /// Entry speed magnitude [m/s]
    pub entry_speed_mps: f64,
    /// Entry flight-path angle [deg], negative is descending
    pub entry_flight_path_deg: f64,
    /// Trust EMA factor for DSFB observers
    pub rho: f64,
    /// Slew threshold for acceleration channels [m/s^3]
    pub slew_threshold_accel: f64,
    /// Slew threshold for gyro channels [rad/s^2]
    pub slew_threshold_gyro: f64,
    /// Penalty scale when slew threshold is exceeded
    pub slew_penalty_gain: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            dt: 0.2,
            t_final: 900.0,
            imu_count: 3,
            seed: 17,
            blackout_upper_m: 80_000.0,
            blackout_lower_m: 40_000.0,
            entry_altitude_m: 120_000.0,
            entry_speed_mps: 7_500.0,
            entry_flight_path_deg: -5.5,
            rho: 0.97,
            slew_threshold_accel: 32.0,
            slew_threshold_gyro: 1.4,
            slew_penalty_gain: 0.75,
        }
    }
}

impl SimConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.dt > 0.0, "dt must be > 0");
        anyhow::ensure!(self.t_final > self.dt, "t_final must be > dt");
        anyhow::ensure!(self.imu_count >= 2, "imu_count must be at least 2");
        anyhow::ensure!(
            self.blackout_upper_m > self.blackout_lower_m,
            "blackout_upper_m must be larger than blackout_lower_m"
        );
        anyhow::ensure!(self.rho > 0.0 && self.rho < 1.0, "rho must be in (0, 1)");
        Ok(())
    }

    pub fn steps(&self) -> usize {
        (self.t_final / self.dt).ceil() as usize
    }
}
