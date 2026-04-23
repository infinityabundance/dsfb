# GUM Uncertainty Budget for DSFB-RF Admissibility Envelopes

**Reference:** JCGM 100:2008 (GUM), ISO/IEC Guide 98-3, IEEE 1764

## Overview

The admissibility envelope radius ρ determines the boundary between Admissible
and Boundary/Violation grammar states. Its uncertainty directly affects episode
precision and false-episode rate. This document defines the GUM-traceable
uncertainty budget for ρ.

## Pre-condition: Wide-Sense Stationarity (WSS)

GUM Type A uncertainty requires observations from a stationary process.
The `stationarity.rs` module verifies this pre-condition using the
Wiener-Khinchin framework before the uncertainty budget is computed.

**Verification checks:**
- Mean stationarity: |μ₁ − μ₂| / max(|μ₁|, |μ₂|) < 20%
- Variance stationarity: |σ₁² − σ₂²| / max(σ₁², σ₂²) < 50%
- Autocorrelation decay: |r(1)/r(0)| < 0.70

If WSS verification fails, the uncertainty budget is flagged as unreliable
and the operator is warned.

## Type A: Statistical Uncertainty

Derived from N independent observations in the healthy calibration window:

```
u_A = σ_healthy / √N
```

For N=100, σ=0.01: u_A = 0.001

## Type B: Systematic Uncertainty Contributors

| Contributor               | Typical u_B  | Source                           |
|---------------------------|-------------|----------------------------------|
| Receiver noise figure     | 0.005       | Manufacturer spec ±0.5 dB        |
| ADC quantization noise    | 0.001       | Q/√12 for 14-bit ADC             |
| Thermal gain drift        | 0.003       | 0.02 dB/°C over ±10°C           |
| LO phase noise floor      | 0.002       | Phase noise spec at offset       |
| IQ imbalance (DC offset)  | 0.002       | Hardware characterization        |

Combined Type B: u_B = √(Σ u_B,i²) ≈ 0.0066

## Combined and Expanded Uncertainty

```
u_c = √(u_A² + u_B²) ≈ √(0.001² + 0.0066²) ≈ 0.0067
U = k · u_c = 3.0 × 0.0067 = 0.020   (coverage factor k=3, 99.7%)
```

## GUM-Derived Envelope Radius

```
ρ_GUM = μ_healthy + U
```

For μ=0.035, U=0.020: ρ_GUM = 0.055

Compare with the ad-hoc 3σ rule: ρ_3σ = μ + 3σ = 0.035 + 0.030 = 0.065

The GUM-derived ρ is typically tighter than the ad-hoc 3σ rule when
Type B contributors are well-characterized, because u_A decreases as
1/√N while the 3σ rule uses the full population σ.

## API Usage

```rust
use dsfb_rf::uncertainty::{compute_budget, UncertaintyConfig};
use dsfb_rf::stationarity::{verify_wss, StationarityConfig};

let healthy_norms: &[f32] = &[/* 100 observations */];

// Step 1: Verify WSS pre-condition
let wss = verify_wss(healthy_norms, &StationarityConfig::default());
let wss_ok = wss.map_or(false, |v| v.is_wss);

// Step 2: Compute GUM budget
let config = UncertaintyConfig::typical_sdr();
let budget = compute_budget(healthy_norms, &config, wss_ok).unwrap();

println!("ρ_GUM = {:.4}", budget.rho_gum);
println!("u_A = {:.4}, u_B = {:.4}, u_c = {:.4}", budget.u_a, budget.u_b_combined, budget.u_c);
println!("WSS verified: {}", budget.wss_verified);
```

## Calibration Integrity

If the calibration window is contaminated (early interference, gain transient),
both u_A and the WSS check will flag the problem:
- u_A will be inflated (high variance from non-stationary data)
- WSS will fail (mean/variance shift between halves)

The operator should re-calibrate with a verified clean window before deployment.

## Non-Claim

This uncertainty budget applies to the evaluated SDR configurations
(USRP B200 class). Type B contributors must be re-characterized for
each deployment platform. No MIL-STD-461G or DO-178C compliance claim
is made from this analysis.
