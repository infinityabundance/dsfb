# Uncertainty budget (GUM JCGM 100:2008)

This document enumerates the uncertainty components that contribute
to DSFB's calibrated envelope radius ρ and downstream grammar
transitions, per the *Guide to the Expression of Uncertainty in
Measurement* (JCGM 100:2008). The budget is implemented in
[`src/uncertainty.rs`](../src/uncertainty.rs) and exposed via the
`UncertaintyComponent` / `combined_standard_uncertainty` /
`expanded_uncertainty` API.

Reporting `ρ = μ + 3σ` as a point estimate without an uncertainty
budget is incompatible with metrological honesty. DSFB's calibration
inherits the measurement uncertainty of the residual stream it is
given, so the budget is **dataset-specific** — each
`docs/<slug>_oracle_protocol.md` specifies the provenance, and the
Phase 9 full-audit run populates the per-dataset budget in
`audit/uncertainty/<slug>_budget.json`.

## Uncertainty model

Combined standard uncertainty of the envelope radius ρ:

```
u_c(ρ)² = Σ u_i²
```

where `u_i` is the standard uncertainty of the `i`-th component
(Type A or Type B per GUM). Expanded uncertainty at coverage factor
`k`:

```
U = k · u_c(ρ)
```

The default reporting coverage is `k = 2` (≈ 95 %
Normal-distribution coverage). `k = 3` (≈ 99.7 %) is available for
higher-criticality reporting — use at the paper-section's discretion.

## Per-dataset uncertainty surface

For each of the ten datasets, the components contributing to the
ρ-uncertainty are:

### Kinematics datasets (KUKA LWR, Panda Gaz, DLR Justin, UR10 Kufieta)

| Component | GUM type | Source |
|---|---|---|
| `identified_parameter_vector_residual_stddev` | A | Least-squares residual of `θ̂` fit in the source paper (Jubien 2014 / Gaz 2019 / Albu-Schäffer et al. / Kufieta 2014) |
| `torque_sensor_noise_floor` | B | Manufacturer datasheet — typically 0.1 % full-scale RMS for link-side sensors; 1–2 % for motor-side current reconstruction |
| `kinematic_measurement_quantisation` | B | Joint-encoder LSB; typically 1 µrad or 0.0002 degrees for research arms |
| `healthy_window_sample_size` | A | Standard error of the mean for the specific calibration window length |

### Balancing datasets (Cheetah 3, iCub push-recovery)

| Component | GUM type | Source |
|---|---|---|
| `imu_allan_variance_floor` | B | IMU datasheet, typically 1e-4 m/s² bias stability |
| `force_torque_sensor_repeatability` | B | ATI / FT sensor datasheet — ≈ 0.1 % FS |
| `whole_body_controller_model_residual` | A | MPC tracking-error baseline over a calibration window |
| `healthy_window_sample_size` | A | SEM of the calibration window |

### PHM datasets (CWRU, IMS, FEMTO-ST)

| Component | GUM type | Source |
|---|---|---|
| `vibration_sensor_noise_floor` | B | Accelerometer datasheet (e.g. IMI 603C01 for CWRU) |
| `healthy_window_statistical_spread` | A | σ of calibration-window HI or envelope amplitude |
| `operating_condition_variability` | B | Load / speed drift across calibration snapshots |

### C-MAPSS (simulation caveat)

Uncertainty for C-MAPSS is simulation-derived and treated as a
nominal-reference rather than a measurement uncertainty. The
`cmapss` budget is flagged as "cross-domain analogue" per
[`cmapss_oracle_protocol.md`](cmapss_oracle_protocol.md).

## Paper §18 uncertainty table

The companion paper's §18 "Uncertainty budget" populates a
per-dataset table with:

1. Component name (matching the `UncertaintyComponent.name` field).
2. GUM type (A / B).
3. Standard uncertainty `u_i` in the residual's units (N·m, N, or
   dimensionless HI).
4. Combined standard uncertainty `u_c`.
5. Expanded uncertainty `U` at `k = 2`.

The table is populated in Phase 9 after the dsfb-gray audit surface
is fully stable and the per-dataset component values are cited from
their upstream documentation sources.

## Reproducibility of the budget

The uncertainty components listed above are **cited**, not measured
by this crate. The crate's `uncertainty` module provides the GUM
*combination* machinery; the component values come from dataset
provenance (manufacturer datasheets, paper fits). Any change to the
cited components should be reflected simultaneously in:

1. The per-dataset oracle-protocol file.
2. `audit/uncertainty/<slug>_budget.json`.
3. The companion paper's §18 table.

This three-way mirror prevents silent uncertainty-budget drift
between code, docs, and paper.
