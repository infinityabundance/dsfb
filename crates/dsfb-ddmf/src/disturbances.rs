use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DisturbanceKind {
    PointwiseBounded {
        d: f64,
    },
    Drift {
        b: f64,
        s_max: f64,
    },
    SlewRateBounded {
        s_max: f64,
    },
    Impulsive {
        amplitude: f64,
        start: usize,
        len: usize,
    },
    PersistentElevated {
        r_nom: f64,
        r_high: f64,
        step_time: usize,
    },
}

pub trait Disturbance {
    fn reset(&mut self);
    fn next(&mut self, n: usize) -> f64;
}

#[derive(Clone, Debug)]
pub struct PointwiseBoundedDisturbance {
    d: f64,
}

impl PointwiseBoundedDisturbance {
    pub fn new(d: f64) -> Self {
        Self { d }
    }
}

impl Disturbance for PointwiseBoundedDisturbance {
    fn reset(&mut self) {}

    fn next(&mut self, _n: usize) -> f64 {
        self.d
    }
}

#[derive(Clone, Debug)]
pub struct DriftDisturbance {
    b: f64,
    s_max: f64,
}

impl DriftDisturbance {
    pub fn new(b: f64, s_max: f64) -> Self {
        Self { b, s_max }
    }
}

impl Disturbance for DriftDisturbance {
    fn reset(&mut self) {}

    fn next(&mut self, n: usize) -> f64 {
        (self.b * n as f64).clamp(-self.s_max, self.s_max)
    }
}

#[derive(Clone, Debug)]
pub struct SlewRateBoundedDisturbance {
    s_max: f64,
    value: f64,
}

impl SlewRateBoundedDisturbance {
    pub fn new(s_max: f64) -> Self {
        Self { s_max, value: 0.0 }
    }
}

impl Disturbance for SlewRateBoundedDisturbance {
    fn reset(&mut self) {
        self.value = 0.0;
    }

    fn next(&mut self, n: usize) -> f64 {
        if n == 0 {
            return self.value;
        }
        self.value += self.s_max;
        self.value
    }
}

#[derive(Clone, Debug)]
pub struct ImpulsiveDisturbance {
    amplitude: f64,
    start: usize,
    len: usize,
}

impl ImpulsiveDisturbance {
    pub fn new(amplitude: f64, start: usize, len: usize) -> Self {
        Self {
            amplitude,
            start,
            len,
        }
    }
}

impl Disturbance for ImpulsiveDisturbance {
    fn reset(&mut self) {}

    fn next(&mut self, n: usize) -> f64 {
        if n >= self.start && n < self.start.saturating_add(self.len) {
            self.amplitude
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug)]
pub struct PersistentElevatedDisturbance {
    r_nom: f64,
    r_high: f64,
    step_time: usize,
}

impl PersistentElevatedDisturbance {
    pub fn new(r_nom: f64, r_high: f64, step_time: usize) -> Self {
        Self {
            r_nom,
            r_high,
            step_time,
        }
    }
}

impl Disturbance for PersistentElevatedDisturbance {
    fn reset(&mut self) {}

    fn next(&mut self, n: usize) -> f64 {
        if n < self.step_time {
            self.r_nom
        } else {
            self.r_high
        }
    }
}

pub fn build_disturbance(kind: &DisturbanceKind) -> Box<dyn Disturbance> {
    match kind {
        DisturbanceKind::PointwiseBounded { d } => Box::new(PointwiseBoundedDisturbance::new(*d)),
        DisturbanceKind::Drift { b, s_max } => Box::new(DriftDisturbance::new(*b, *s_max)),
        DisturbanceKind::SlewRateBounded { s_max } => {
            Box::new(SlewRateBoundedDisturbance::new(*s_max))
        }
        DisturbanceKind::Impulsive {
            amplitude,
            start,
            len,
        } => Box::new(ImpulsiveDisturbance::new(*amplitude, *start, *len)),
        DisturbanceKind::PersistentElevated {
            r_nom,
            r_high,
            step_time,
        } => Box::new(PersistentElevatedDisturbance::new(
            *r_nom, *r_high, *step_time,
        )),
    }
}

impl DisturbanceKind {
    pub fn disturbance_type(&self) -> &'static str {
        match self {
            DisturbanceKind::PointwiseBounded { .. } => "pointwise_bounded",
            DisturbanceKind::Drift { .. } => "drift",
            DisturbanceKind::SlewRateBounded { .. } => "slew_rate_bounded",
            DisturbanceKind::Impulsive { .. } => "impulsive",
            DisturbanceKind::PersistentElevated { .. } => "persistent_elevated",
        }
    }

    pub fn regime_label(&self) -> &'static str {
        match self {
            DisturbanceKind::PointwiseBounded { d } if d.abs() <= 0.15 => "bounded_nominal",
            DisturbanceKind::PointwiseBounded { .. } => "persistent_elevated",
            DisturbanceKind::Drift { .. } => "persistent_elevated",
            DisturbanceKind::SlewRateBounded { .. } => "unbounded",
            DisturbanceKind::Impulsive { .. } => "impulsive",
            DisturbanceKind::PersistentElevated { .. } => "persistent_elevated",
        }
    }

    pub fn recovery_target(&self, nominal_bound: f64) -> Option<f64> {
        match self {
            DisturbanceKind::PointwiseBounded { d } if d.abs() <= 0.15 => Some(d.abs()),
            DisturbanceKind::Impulsive { .. } => Some(nominal_bound.abs()),
            _ => None,
        }
    }

    pub fn recovery_search_start(&self) -> Option<usize> {
        match self {
            DisturbanceKind::PointwiseBounded { d } if d.abs() <= 0.15 => Some(0),
            DisturbanceKind::Impulsive { start, len, .. } => Some(start.saturating_add(*len)),
            _ => None,
        }
    }

    pub fn monte_carlo_columns(&self) -> (f64, f64, f64, usize, usize) {
        match self {
            DisturbanceKind::PointwiseBounded { d } => (d.abs(), 0.0, 0.0, 0, 0),
            DisturbanceKind::Drift { b, s_max } => (0.0, *b, *s_max, 0, 0),
            DisturbanceKind::SlewRateBounded { s_max } => (0.0, 0.0, *s_max, 0, 0),
            DisturbanceKind::Impulsive {
                amplitude,
                start,
                len,
            } => (amplitude.abs(), 0.0, 0.0, *start, *len),
            DisturbanceKind::PersistentElevated {
                r_nom,
                r_high,
                step_time,
            } => (r_high.abs(), *r_nom, 0.0, *step_time, 0),
        }
    }

    pub fn channelized(&self, key: usize) -> Self {
        let scale = 1.0 + 0.03 * key as f64;
        match self {
            DisturbanceKind::PointwiseBounded { d } => Self::PointwiseBounded { d: d * scale },
            DisturbanceKind::Drift { b, s_max } => Self::Drift {
                b: b * scale,
                s_max: s_max * scale,
            },
            DisturbanceKind::SlewRateBounded { s_max } => Self::SlewRateBounded {
                s_max: s_max * scale,
            },
            DisturbanceKind::Impulsive {
                amplitude,
                start,
                len,
            } => Self::Impulsive {
                amplitude: amplitude * scale,
                start: start.saturating_add(key % 3),
                len: *len,
            },
            DisturbanceKind::PersistentElevated {
                r_nom,
                r_high,
                step_time,
            } => Self::PersistentElevated {
                r_nom: r_nom * scale,
                r_high: r_high * scale,
                step_time: step_time.saturating_add(key % 4),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_disturbance, DisturbanceKind};

    #[test]
    fn impulsive_disturbance_is_zero_outside_window() {
        let mut disturbance = build_disturbance(&DisturbanceKind::Impulsive {
            amplitude: 2.0,
            start: 3,
            len: 2,
        });

        assert_eq!(disturbance.next(2), 0.0);
        assert_eq!(disturbance.next(3), 2.0);
        assert_eq!(disturbance.next(5), 0.0);
    }

    #[test]
    fn slew_rate_bounded_disturbance_accumulates_without_magnitude_bound() {
        let mut disturbance = build_disturbance(&DisturbanceKind::SlewRateBounded { s_max: 0.25 });
        let _ = disturbance.next(0);
        let d1 = disturbance.next(1);
        let d2 = disturbance.next(2);
        let d8 = disturbance.next(8);

        assert!((d2 - d1 - 0.25).abs() < 1e-12);
        assert!(d8 > d2);
    }
}
