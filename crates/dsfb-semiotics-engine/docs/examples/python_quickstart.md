# Python Quickstart

The crate ships a nested PyO3 package under [`python/`](../../python/) for researchers who want a
thin Python or Jupyter entrypoint without reimplementing the engine logic.

Build/install with maturin:

```bash
cd crates/dsfb-semiotics-engine/python
maturin develop
```

Quickstart:

```python
import dsfb_engine

summary = dsfb_engine.run_scenario("nominal_stable")
trace = dsfb_engine.run_array([0.04, 0.08, 0.12, 0.20])
```

The package currently exposes:

- `SemioticsEngine(...)` for bounded online use
- `run_scenario(scenario_id)` for one deterministic synthetic scenario
- `run_csv(observed_csv, predicted_csv, ...)` for the observed/predicted CSV path
- `run_array(values)` for small array-like residual traces

Example script:

- [`python/examples/quickstart.py`](../../python/examples/quickstart.py)

This is a convenience binding layer over the deterministic Rust implementation. It does not alter
the crate’s conservative scientific posture.
