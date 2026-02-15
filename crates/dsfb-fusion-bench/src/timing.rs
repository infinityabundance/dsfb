use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct TimingAccumulator {
    pub solve_time: Duration,
    pub total_time: Duration,
    pub steps: usize,
}

impl TimingAccumulator {
    pub fn observe(&mut self, solve_time: Duration, total_time: Duration) {
        self.solve_time += solve_time;
        self.total_time += total_time;
        self.steps += 1;
    }

    pub fn avg_solve_us(&self) -> f64 {
        if self.steps == 0 {
            return 0.0;
        }
        (self.solve_time.as_secs_f64() * 1e6) / self.steps as f64
    }

    pub fn avg_total_us(&self) -> f64 {
        if self.steps == 0 {
            return 0.0;
        }
        (self.total_time.as_secs_f64() * 1e6) / self.steps as f64
    }
}
