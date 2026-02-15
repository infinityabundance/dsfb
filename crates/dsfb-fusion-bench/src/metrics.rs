#[derive(Debug, Clone)]
pub struct MethodMetrics {
    pub peak_err: f64,
    pub rms_err: f64,
    pub false_downweight_rate: Option<f64>,
}

#[derive(Debug, Default, Clone)]
pub struct MetricsAccumulator {
    peak_err: f64,
    sum_sq: f64,
    count: usize,
    false_downweight_count: usize,
    false_downweight_total: usize,
    expects_weights: bool,
}

impl MetricsAccumulator {
    pub fn new(expects_weights: bool) -> Self {
        Self {
            expects_weights,
            ..Self::default()
        }
    }

    pub fn observe(
        &mut self,
        err_norm: f64,
        group_weights: Option<&[f64]>,
        corruption_active: bool,
    ) {
        self.peak_err = self.peak_err.max(err_norm);
        self.sum_sq += err_norm * err_norm;
        self.count += 1;

        if self.expects_weights && !corruption_active {
            if let Some(weights) = group_weights {
                for &w in weights {
                    self.false_downweight_total += 1;
                    if w < 0.9 {
                        self.false_downweight_count += 1;
                    }
                }
            }
        }
    }

    pub fn finalize(&self) -> MethodMetrics {
        let rms_err = if self.count > 0 {
            (self.sum_sq / self.count as f64).sqrt()
        } else {
            0.0
        };

        let false_downweight_rate = if self.expects_weights {
            if self.false_downweight_total > 0 {
                Some(self.false_downweight_count as f64 / self.false_downweight_total as f64)
            } else {
                Some(0.0)
            }
        } else {
            None
        };

        MethodMetrics {
            peak_err: self.peak_err,
            rms_err,
            false_downweight_rate,
        }
    }
}
