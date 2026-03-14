#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTrajectory {
    pub id: String,
    pub states: Vec<i32>,
}

impl StateTrajectory {
    pub fn new(id: impl Into<String>, states: Vec<i32>) -> Self {
        Self {
            id: id.into(),
            states,
        }
    }
}

pub fn fine_regime(state: i32) -> &'static str {
    match state.cmp(&0) {
        std::cmp::Ordering::Less => "negative",
        std::cmp::Ordering::Equal => "zero",
        std::cmp::Ordering::Greater => "positive",
    }
}

pub fn coarse_regime(fine: &str) -> &'static str {
    match fine {
        "positive" => "high",
        _ => "low",
    }
}

pub fn trajectories() -> Vec<StateTrajectory> {
    vec![
        StateTrajectory::new("constant_negative", vec![-2, -2, -2, -2]),
        StateTrajectory::new("alternating", vec![-1, 1, -1, 1, -1, 1]),
        StateTrajectory::new("block_constant", vec![-2, -2, 0, 0, 2, 2, 2]),
        StateTrajectory::new("periodic", vec![-1, 0, 1, 0, -1, 0, 1, 0]),
        StateTrajectory::new("stabilizing", vec![-2, -1, 0, 0, 0, 0]),
    ]
}
