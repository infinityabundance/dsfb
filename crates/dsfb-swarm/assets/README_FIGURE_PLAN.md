# DSFB-Swarm Figure Plan

The crate writes a stable figure set into `output-dsfb-swarm/<timestamp>/figures/`.

## Figure inventory

1. `lambda2_timeseries.png`
   - Shows `lambda_2(t)` across scenarios.
   - Main paper-facing claim: connectivity collapse is visible in the spectrum.

2. `residual_timeseries.png`
   - Upper panel: observed versus predicted `lambda_2(t)`.
   - Lower panel: scalar residual `r_lambda2(t)` with its envelope.
   - Main claim: residual-centered monitoring detects structure before a raw threshold alone.

3. `drift_slew.png`
   - Plots residual drift and residual slew.
   - Main claim: persistent negative residual drift is an early finite-time detectability signal.

4. `trust_evolution.png`
   - Tracks trust over affected nodes under trust-gated attenuation.
   - Main claim: degraded or adversarial interactions are suppressed before global failure.

5. `baseline_comparison.png`
   - Compares lead time against state-norm, disagreement-energy, and raw-`lambda_2` baselines.
   - Main claim: predictor-residual diagnostics outperform simpler alarms on the same runs.

6. `scaling_curves.png`
   - Runtime versus swarm size under benchmark sweeps.
   - Main claim: spectral monitoring remains usable as the network grows.

7. `noise_stress_curves.png`
   - TPR/FPR trends against deterministic noise amplitude.
   - Main claim: the residual stack remains informative under bounded disturbance.

8. `multimode_comparison.png`
   - Scalar `lambda_2` monitoring versus multi-mode residual stacks.
   - Main claim: multi-mode monitoring detects anomalies earlier or more reliably than `lambda_2` alone.

9. `topology_snapshots.png`
   - Pre-anomaly, onset, and late topology snapshots.
   - Main claim: spectral changes correspond to interpretable graph-topology changes.
