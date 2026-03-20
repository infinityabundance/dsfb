# Dashboard Replay Example

The crate includes a deterministic `ratatui` replay dashboard for synthetic and CSV-driven runs.
The dashboard consumes typed engine/evaluation replay events; it does not recompute the science in
the UI layer.

Replay one synthetic scenario:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario nominal_stable \
  --dashboard-replay \
  --dashboard-max-frames 4
```

Replay one CSV fixture:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --input-mode csv \
  --observed-csv crates/dsfb-semiotics-engine/tests/fixtures/observed_fixture.csv \
  --predicted-csv crates/dsfb-semiotics-engine/tests/fixtures/predicted_fixture.csv \
  --scenario-id fixture_csv \
  --time-column time \
  --dashboard-replay \
  --dashboard-scenario fixture_csv \
  --dashboard-max-frames 2
```

The replay panels show:

- current scenario / stream identity
- residual norm, drift norm, and slew norm
- projected sign coordinates
- syntax headline, grammar state, and semantic disposition
- admissibility audit counts and comparator alarms
- an event / transition log derived from the replay stream
