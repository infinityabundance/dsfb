# Reviewer Summary

On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.31904.

Demo B was not run in this command. Run `cargo run -- run-demo-b --output <dir>` or `cargo run -- run-all --output <dir>` to generate the fixed-budget allocation study.

What is now decision-clean: host-realistic mode exists, stronger baselines are included, multiple deterministic scenarios are reported, ablations isolate cue dependence, Demo B is fixed-budget across multiple policies, and attachability/cost are explicit.

What is still blocked: synthetic scene scope, lack of measured GPU benchmarks, and mixed outcomes against the strongest heuristic baseline on some scenarios.

This crate is ready for internal technical evaluation and funding diligence. It is not presented as a production-readiness or licensing-closing proof.
