# Live Drop-In Example

The crate includes a bounded online example at `examples/live_drop_in.rs`.

Run it with:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example live_drop_in
```

What it demonstrates:

- one-sample-at-a-time pushing through the bounded online engine
- fixed-capacity history only on the live path
- immediate query of syntax, grammar, semantics, and trust after each push
- an operator-readable trace that a C++ controls engineer or systems integrator can follow quickly

The example keeps the scientific posture conservative. The printed trust value is a deterministic
deployment-oriented interface derived from grammar severity. It is not a probabilistic confidence
estimate or a field-validated control law.
