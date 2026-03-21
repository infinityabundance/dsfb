# State Replay Workflow

The bounded live engine can serialize a versioned snapshot, restore it later, and replay exactly one more sample.

Example support workflow:

1. capture a snapshot from the bounded engine in the process that observed the surprising transition
2. save the binary blob to a file such as `/tmp/live.dsfb`
3. replay the next sample with:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-state-replay -- \
  --snapshot-in /tmp/live.dsfb \
  --sample-time 12.0 \
  --sample-values 0.12
```

4. inspect the emitted syntax, grammar, semantics, and trust outputs

This is a state-exact replay aid under the documented build and numeric backend. It is not a universal cross-platform bit-exactness claim.
