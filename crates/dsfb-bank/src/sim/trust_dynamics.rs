#[derive(Debug, Clone, PartialEq)]
pub struct TrustStep {
    pub iteration: usize,
    pub state_id: String,
    pub next_state_id: String,
    pub trust_value: f64,
    pub next_trust_value: f64,
    pub fixed_point_flag: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustOrbit {
    pub orbit_id: String,
    pub steps: Vec<TrustStep>,
    pub stabilization_iteration: usize,
}

pub fn descending_orbit(id: &str, start: i32, extra_tail: usize) -> TrustOrbit {
    let mut states = Vec::new();
    let mut current = start.max(0);
    loop {
        states.push(current);
        if current == 0 {
            break;
        }
        current -= 1;
    }
    for _ in 0..extra_tail {
        states.push(0);
    }
    orbit_from_states(id, &states)
}

pub fn neutral_cycle_orbit(id: &str, len: usize) -> TrustOrbit {
    let mut states = Vec::with_capacity(len + 1);
    for index in 0..=len {
        let state = if index % 2 == 0 { 10 } else { 11 };
        states.push(state);
    }
    orbit_from_states(id, &states)
}

pub fn custom_orbit(id: &str, states: &[i32]) -> TrustOrbit {
    orbit_from_states(id, states)
}

fn orbit_from_states(id: &str, states: &[i32]) -> TrustOrbit {
    let mut steps = Vec::new();
    for iteration in 0..states.len().saturating_sub(1) {
        let state = states[iteration];
        let next_state = states[iteration + 1];
        steps.push(TrustStep {
            iteration,
            state_id: format!("s{state}"),
            next_state_id: format!("s{next_state}"),
            trust_value: trust_value(state),
            next_trust_value: trust_value(next_state),
            fixed_point_flag: state == next_state,
        });
    }
    let stabilization_iteration = steps
        .iter()
        .find(|step| step.fixed_point_flag)
        .map(|step| step.iteration)
        .unwrap_or_else(|| steps.last().map(|step| step.iteration).unwrap_or(0));
    TrustOrbit {
        orbit_id: id.to_string(),
        steps,
        stabilization_iteration,
    }
}

pub fn trust_value(state: i32) -> f64 {
    if state >= 10 {
        1.0
    } else {
        state.max(0) as f64
    }
}
