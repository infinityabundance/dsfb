# Dashboard Replay Example

The crate includes a deterministic `ratatui` replay dashboard for synthetic runs and a first-class
CSV live replay mode for observed/predicted CSV inputs. The dashboard consumes typed
engine/evaluation replay events; it does not recompute the science in the UI layer.

Replay one synthetic scenario:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario nominal_stable \
  --dashboard-replay \
  --dashboard-max-frames 4
```

Replay one CSV fixture through the dedicated CSV live replay driver:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --input-mode csv \
  --observed-csv crates/dsfb-semiotics-engine/tests/fixtures/observed_fixture.csv \
  --predicted-csv crates/dsfb-semiotics-engine/tests/fixtures/predicted_fixture.csv \
  --scenario-id fixture_csv \
  --time-column time \
  --dashboard-replay-csv \
  --dashboard-playback-speed 1.0 \
  --dashboard-scenario fixture_csv \
  --dashboard-max-frames 2
```

The CSV replay top bar shows:

- `REPLAY MODE: CSV`
- the source file pair or scenario id
- current replay time
- playback speed
- paused / running / ended status

The replay panels show:

- current scenario / stream identity
- residual norm, drift norm, and slew norm
- projected sign coordinates
- syntax headline, grammar state, and semantic disposition
- admissibility audit counts and comparator alarms
- event markers for syntax, grammar, semantic, comparator, and trust-threshold transitions
- an event / transition log derived from the replay stream

The CSV replay driver exposes deterministic play/pause, single-step, playback-rate, and
end-of-stream state in code. The CLI surface is intentionally conservative: it prints a replay
walkthrough rather than claiming a fully interactive console control loop.
