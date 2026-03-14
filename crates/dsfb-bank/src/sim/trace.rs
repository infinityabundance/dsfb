use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTrace {
    pub id: String,
    pub events: Vec<char>,
}

impl EventTrace {
    pub fn new(id: impl Into<String>, events: Vec<char>) -> Self {
        Self {
            id: id.into(),
            events,
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn concat(&self, suffix: &Self, id: impl Into<String>) -> Self {
        let mut events = self.events.clone();
        events.extend_from_slice(&suffix.events);
        Self::new(id, events)
    }

    pub fn prefix(&self, len: usize, id: impl Into<String>) -> Self {
        Self::new(id, self.events.iter().take(len).copied().collect())
    }

    pub fn suffix(&self, len: usize, id: impl Into<String>) -> Self {
        let start = self.events.len().saturating_sub(len);
        Self::new(id, self.events.iter().skip(start).copied().collect())
    }

    pub fn as_string(&self) -> String {
        self.events.iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCodebook {
    by_trace: BTreeMap<String, u32>,
    by_code: BTreeMap<u32, String>,
}

impl TraceCodebook {
    pub fn from_traces(traces: &[EventTrace]) -> Self {
        let mut by_trace = BTreeMap::new();
        let mut by_code = BTreeMap::new();
        for (index, trace) in traces.iter().enumerate() {
            let trace_string = trace.as_string();
            let code = 1000 + index as u32;
            by_trace.insert(trace_string.clone(), code);
            by_code.insert(code, trace_string);
        }
        Self { by_trace, by_code }
    }

    pub fn observation_code(&self, trace: &EventTrace) -> Option<u32> {
        self.by_trace.get(&trace.as_string()).copied()
    }

    pub fn reconstruct(&self, code: u32) -> Option<EventTrace> {
        self.by_code
            .get(&code)
            .map(|trace| EventTrace::new(format!("reconstructed_{code}"), trace.chars().collect()))
    }
}

pub fn sample_traces() -> Vec<EventTrace> {
    vec![
        EventTrace::new("tau", vec!['a', 'b']),
        EventTrace::new("sigma", vec!['b', 'c', 'a']),
        EventTrace::new("rho", vec!['c']),
        EventTrace::new("periodic", vec!['a', 'b', 'a', 'b', 'a', 'b']),
    ]
}

pub fn periodic_window_observation(trace: &EventTrace, window: usize) -> Vec<String> {
    let mut observations = Vec::with_capacity(trace.events.len());
    for index in 0..trace.events.len() {
        let start = index.saturating_add(1).saturating_sub(window);
        let observation = trace.events[start..=index].iter().collect();
        observations.push(observation);
    }
    observations
}
