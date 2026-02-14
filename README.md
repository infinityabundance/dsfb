# DSFB - Drift-Slew Fusion Bootstrap

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/notebooks/dsfb_simulation.ipynb)

A Rust implementation of the Drift-Slew Fusion Bootstrap (DSFB) algorithm for trust-adaptive nonlinear state estimation.

## Overview

DSFB is a state estimation algorithm that tracks position (φ), velocity/drift (ω), and acceleration/slew (α) across multiple measurement channels with adaptive trust weighting. The algorithm dynamically adjusts trust weights for each channel based on exponential moving averages (EMA) of residuals, making it robust to impulse disturbances and measurement anomalies.

### Key Features

- **Trust-adaptive fusion**: Automatically down-weights unreliable measurement channels
- **Drift-slew dynamics**: Tracks position, velocity (drift), and acceleration (slew)
- **O(M) complexity**: Efficient per-step computation for M channels
- **Deterministic**: Reproducible results with seed control
- **Pure Rust**: No external C dependencies

## Algorithm

The DSFB algorithm operates in discrete time with the following steps:

### Predict
```
φ⁻ = φ + ω·dt
ω⁻ = ω + α·dt
α⁻ = α
```

### Update Trust Weights
```
rₖ = yₖ - h(φ⁻)              # Residuals
sₖ = ρ·sₖ + (1-ρ)·|rₖ|       # EMA residuals
w̃ₖ = 1 / (σ₀ + sₖ)           # Raw trust weights
wₖ = w̃ₖ / Σⱼw̃ⱼ              # Normalized weights
```

### Correct
```
R = Σₖ wₖ·rₖ                 # Aggregate residual
φ = φ⁻ + k_φ·R
ω = ω⁻ + k_ω·R
α = α⁻ + k_α·R
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dsfb = "0.1"
```

Or install from source:

```bash
git clone https://github.com/infinityabundance/dsfb
cd dsfb
cargo build --release
```

## Usage

### Basic Example

```rust
use dsfb::{DsfbObserver, DsfbParams, DsfbState};

// Configure parameters
let params = DsfbParams::new(
    0.5,  // k_phi: position gain
    0.1,  // k_omega: velocity gain
    0.01, // k_alpha: acceleration gain
    0.95, // rho: EMA smoothing (0 < ρ < 1)
    0.1,  // sigma0: trust softness
);

// Create observer with 2 channels
let mut observer = DsfbObserver::new(params, 2);

// Initialize state
observer.init(DsfbState::new(0.0, 0.5, 0.0));

// Process measurements
let dt = 0.01;
let measurements = vec![1.0, 1.05];
let state = observer.step(&measurements, dt);

println!("φ={}, ω={}, α={}", state.phi, state.omega, state.alpha);

// Check trust weights
let w0 = observer.trust_weight(0);
let w1 = observer.trust_weight(1);
println!("Trust weights: {}, {}", w0, w1);
```

### Running the Simulation

The repository includes a complete simulation harness that demonstrates DSFB performance against baseline methods:

```bash
# Run simulation and generate CSV output
cargo run --example drift_impulse

# Output will be written to: out/sim.csv
```

The simulation compares three methods:
1. **Mean Fusion**: Simple average of measurements
2. **Freq-Only Observer**: Observer without acceleration state
3. **DSFB Observer**: Full drift-slew fusion with trust adaptation

Simulation scenario:
- Two measurement channels
- Channel 2 has linear drift and impulse disturbance
- Impulse occurs at t=3.0s for 1.0s duration
- Metrics: RMS error, peak error during impulse, recovery time

### Visualizing Results

Use the Jupyter notebook to visualize simulation results:

```bash
# First, run the simulation to generate data
cargo run --example drift_impulse

# Then open the notebook
jupyter notebook notebooks/dsfb_simulation.ipynb
```

Or use Google Colab:
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/notebooks/dsfb_simulation.ipynb)

The notebook displays:
- Position estimates vs ground truth
- Error curves for all methods
- Trust weight adaptation over time
- EMA residual tracking
- Performance metrics table

## API Reference

### `DsfbState`

State vector for the observer:
- `phi: f64` - Position/phase
- `omega: f64` - Velocity/frequency (drift)
- `alpha: f64` - Acceleration/slew

### `DsfbParams`

Algorithm parameters:
- `k_phi: f64` - Position correction gain
- `k_omega: f64` - Velocity correction gain
- `k_alpha: f64` - Acceleration correction gain
- `rho: f64` - EMA smoothing factor (0 < ρ < 1, typical: 0.95)
- `sigma0: f64` - Trust softness parameter (typical: 0.1)

### `DsfbObserver`

Main observer struct:
- `new(params: DsfbParams, channels: usize) -> Self` - Create observer
- `init(&mut self, initial_state: DsfbState)` - Set initial state
- `step(&mut self, measurements: &[f64], dt: f64) -> DsfbState` - Process one time step
- `state(&self) -> DsfbState` - Get current state
- `trust_weight(&self, channel: usize) -> f64` - Get trust weight for channel
- `ema_residual(&self, channel: usize) -> f64` - Get EMA residual for channel

## Testing

```bash
# Run unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_observer_creation
```

## Performance

- **Time complexity**: O(M) per step for M channels
- **Space complexity**: O(M) for channel statistics
- **Deterministic**: Given fixed seed, produces identical results

## Citation

If you use DSFB in your research, please cite:

```bibtex
@software{dsfb2026,
  author = {de Beer, Riaan},
  title = {DSFB: Drift-Slew Fusion Bootstrap},
  year = {2026},
  url = {https://github.com/infinityabundance/dsfb}
}
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## References

For theoretical background, see:
- [Slew-Aware Trust-Adaptive Nonlinear State Estimation](docs/Slew-Aware%20Trust-Adaptive%20Nonlinear%20State%20Estimation.pdf)

## Repository Structure

```
dsfb/
├── Cargo.toml              # Package configuration
├── src/
│   ├── lib.rs              # Public API
│   ├── observer.rs         # DSFB observer implementation
│   ├── state.rs            # State representation
│   ├── params.rs           # Parameters
│   ├── trust.rs            # Trust weight calculations
│   └── sim.rs              # Simulation harness
├── examples/
│   └── drift_impulse.rs    # Example simulation
├── notebooks/
│   └── dsfb_simulation.ipynb  # Visualization notebook
├── docs/                   # Documentation
├── README.md               # This file
├── LICENSE                 # Apache 2.0 license
└── CITATION.cff            # Citation metadata
```
