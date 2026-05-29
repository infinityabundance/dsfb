//! Mass/energy-balance witnesses and control-action context for *instrumented* datasets.
//!
//! The 20 public slices are anonymised (generic columns, no units), so a physics balance cannot be
//! computed from them. The instrumented demonstrators in `data/instrumented/` ship a JSON roles
//! sidecar (`*.roles.json`) that declares variable roles (measured / manipulated / controlled),
//! units, and an explicit balance equation. This module:
//!
//!  1. parses that sidecar ([`RolesDoc`]);
//!  2. recomputes the balance-closure residual from the *raw* columns ([`balance_residual`]) for the
//!     documented balance types (`mass_three_tank`, `energy_cstr`, `mass_quad_tank`, `energy_csth`,
//!     and `mass_tank_volume` for real SCADA level+flow data, with optional metered outflows) -- so
//!     the witness is the framework computing the documented physics from instruments, not a
//!     precomputed column; and
//!  3. exposes the manipulated variables for control-action context ([`RolesDoc::manipulated`]).
//!
//! The closure stream is fed into the ordinary DSFB grammar as a `ProcessStructure` witness detector
//! (see `cli::run_balance_witness`): a non-zero closure is admitted as a residual stream exactly like
//! a statistical residual, so "mass/energy imbalance" becomes court-admissible residual evidence. The
//! witness detects the *sustained shift* of the closure relative to its baseline window (a real
//! balance carries a model/measurement offset; this is not a zero-closure claim).

use crate::data::DataMatrix;
use serde::Deserialize;
use std::path::Path;

/// One variable's declared role and units (from the roles sidecar).
#[derive(Debug, Deserialize)]
pub struct VarRole {
    pub name: String,
    /// "measured" | "manipulated" | "controlled".
    pub role: String,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub quantity: String,
}

/// Parsed `*.roles.json` sidecar. `balance` and `control_action` are kept as raw JSON because their
/// parameter sets differ per balance type; the typed readers below pull what each type needs.
#[derive(Debug, Deserialize)]
pub struct RolesDoc {
    pub dataset: String,
    #[serde(default)]
    pub variables: Vec<VarRole>,
    pub balance: serde_json::Value,
    #[serde(default)]
    pub control_action: Option<serde_json::Value>,
    #[serde(default)]
    pub fault: Option<serde_json::Value>,
}

impl RolesDoc {
    /// Load and parse a roles sidecar.
    pub fn load(path: &Path) -> Result<Self, String> {
        let s =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        serde_json::from_str(&s).map_err(|e| format!("parse {}: {e}", path.display()))
    }

    /// Names of the manipulated (controller-output) variables — the control-action signals.
    pub fn manipulated(&self) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|v| v.role == "manipulated")
            .map(|v| v.name.as_str())
            .collect()
    }
}

/// Recompute the balance-closure residual per sample from the raw columns, dispatching on
/// `balance.type`. `residual[0]` is 0 (the closure needs a one-step difference). The result is in the
/// engineering units of the balance (cm^3/s for the three-tank mass balance; J/min for the CSTR
/// energy balance). Deterministic; pure function of the matrix + roles.
pub fn balance_residual(m: &DataMatrix, roles: &RolesDoc) -> Result<Vec<f64>, String> {
    let b = &roles.balance;
    let btype = b
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or("balance.type missing")?;
    let f = |key: &str| -> Result<f64, String> {
        b.get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("balance.{key} missing/!number"))
    };
    let col = |name: &str| -> Result<usize, String> {
        m.var_names
            .iter()
            .position(|h| h == name)
            .ok_or_else(|| format!("column '{name}' not found"))
    };
    let named_col = |key: &str| -> Result<usize, String> {
        let n = b
            .get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("balance.{key} missing"))?;
        col(n)
    };
    let mut res = vec![0.0f64; m.n_samples];
    match btype {
        "mass_three_tank" => {
            let area = f("tank_area_cm2")?;
            let c12 = f("c12")?;
            let c23 = f("c23")?;
            let dt = f("dt_s")?;
            let lv = b
                .get("levels")
                .and_then(|v| v.as_array())
                .ok_or("balance.levels missing")?;
            let l1 = col(lv[0].as_str().ok_or("levels[0]")?)?;
            let l2 = col(lv[1].as_str().ok_or("levels[1]")?)?;
            let l3 = col(lv[2].as_str().ok_or("levels[2]")?)?;
            // Torricelli inter-tank flow q = c * sgn(dh) * sqrt(|dh|).
            let tor = |c: f64, dh: f64| c * dh.signum() * dh.abs().sqrt();
            for (k, rk) in res.iter_mut().enumerate().skip(1) {
                let (r0, rp) = (m.row(k), m.row(k - 1));
                let dh2 = (r0[l2] - rp[l2]) / dt;
                // Closure = accumulation - net inter-tank flow; nonzero => unaccounted flow (leak).
                *rk = area * dh2 - (tor(c12, r0[l1] - r0[l2]) - tor(c23, r0[l2] - r0[l3]));
            }
        }
        "energy_cstr" => {
            let v = f("V_L")?;
            let rho_cp = f("rho_cp_J_per_LK")?;
            let neg_dhr = f("neg_dHr_J_per_mol")?;
            let k0 = f("k0")?;
            let eor = f("EoverR_K")?;
            let feed_flow = f("F_L_per_min")?;
            let rcc = f("rho_cp_coolant_J_per_LK")?;
            let dt = f("dt_min")?;
            let (tcol, ca, ft) = (
                named_col("reactor_temp")?,
                named_col("reactor_conc")?,
                named_col("feed_temp")?,
            );
            let (fc, tci, tco) = (
                named_col("coolant_flow")?,
                named_col("coolant_temp_in")?,
                named_col("coolant_temp_out")?,
            );
            for (k, rk) in res.iter_mut().enumerate().skip(1) {
                let (r0, rp) = (m.row(k), m.row(k - 1));
                let t = r0[tcol];
                let dt_dt = (r0[tcol] - rp[tcol]) / dt;
                let q_cool = r0[fc] * rcc * (r0[tco] - r0[tci]); // heat removed by coolant
                let rxn = neg_dhr * k0 * (-eor / t).exp() * r0[ca] * v; // heat generated by reaction
                                                                        // Closure = accumulation - (convective in/out + reaction - cooling).
                *rk = rho_cp * v * dt_dt - (feed_flow * rho_cp * (r0[ft] - t) + rxn - q_cool);
            }
        }
        "mass_quad_tank" => {
            // Johansson quadruple-tank, tank-1 mass balance:
            //   A1 dh1/dt = a3 sqrt(2g h3) - a1 sqrt(2g h1) + gamma1 k1 v1.
            // Closure nonzero => unaccounted flow (the unmeasured leak in tank 1).
            let a1_area = f("A1")?;
            let a1 = f("a1")?;
            let a3 = f("a3")?;
            let g = f("g")?;
            let k1 = f("k1")?;
            let gamma1 = f("gamma1")?;
            let dt = f("dt_s")?;
            let (l1, l3, vv1) = (
                named_col("level_1")?,
                named_col("level_3")?,
                named_col("v1")?,
            );
            let sq = |x: f64| x.max(0.0).sqrt();
            for (k, rk) in res.iter_mut().enumerate().skip(1) {
                let (r0, rp) = (m.row(k), m.row(k - 1));
                let dh1 = (r0[l1] - rp[l1]) / dt;
                *rk = a1_area * dh1
                    - (a3 * sq(2.0 * g * r0[l3]) - a1 * sq(2.0 * g * r0[l1])
                        + gamma1 * k1 * r0[vv1]);
            }
        }
        "energy_csth" => {
            // Continuous stirred tank heater, energy balance:
            //   rho_cp V dT/dt = rho_cp Fc (Tc - T) + Q_steam - h_amb (T - Tamb).
            // Closure nonzero => unmodelled loss (the insulation failure), keyed on sustained shift.
            let v = f("V_L")?;
            let rho_cp = f("rho_cp_kJ_per_LK")?;
            let h_amb = f("h_amb_kJ_per_minK")?;
            let tamb = f("Tamb_degC")?;
            let dt = f("dt_min")?;
            let (tcol, tci, fci, qs) = (
                named_col("tank_temp")?,
                named_col("temp_cold_in")?,
                named_col("inflow_cold")?,
                named_col("steam_duty")?,
            );
            for (k, rk) in res.iter_mut().enumerate().skip(1) {
                let (r0, rp) = (m.row(k), m.row(k - 1));
                let t = r0[tcol];
                let dt_dt = (r0[tcol] - rp[tcol]) / dt;
                *rk = rho_cp * v * dt_dt
                    - (rho_cp * r0[fci] * (r0[tci] - t) + r0[qs] - h_amb * (t - tamb));
            }
        }
        "mass_tank_volume" => {
            // Generic storage-tank volume (mass) balance for real SCADA level+flow data:
            //   area dL/dt = sum(metered inflows) - sum(metered outflows) - unmetered demand.
            // Two regimes, both keyed on the *sustained shift / sharp slew* of the closure:
            //   * inflows only (BATADAL/C-Town): outflow is unmetered district demand, so the closure
            //     equals minus that demand -- a bounded diurnal signal; a spoofed level or pump flow
            //     breaks it.
            //   * inflows AND outflows (SWaT stage-1 tank T101): both legs metered, so the closure is
            //     ~0 under normal control; a sensor-spoofing attack that freezes/biases the level while
            //     the flow meters keep moving makes dL/dt contradict (in - out) and the closure jumps.
            // `flow_to_vol_per_dt` reconciles the flow unit to a level change per step (e.g. LPS->m^3/h
            // is 3.6; for SWaT it subsumes the tank area and the aggregation block, calibrated from the
            // normal run). `outflows` is optional (absent => the inflows-only regime).
            let area = f("area_m2")?;
            let dt = f("dt")?;
            let level = named_col("level")?;
            let factor = f("flow_to_vol_per_dt")?;
            let cols_of = |key: &str, required: bool| -> Result<Vec<usize>, String> {
                let arr = match b.get(key).and_then(|v| v.as_array()) {
                    Some(a) => a,
                    None if !required => return Ok(Vec::new()),
                    None => return Err(format!("balance.{key} missing")),
                };
                let mut cs = Vec::with_capacity(arr.len());
                for v in arr {
                    cs.push(col(v
                        .as_str()
                        .ok_or_else(|| format!("balance.{key}[] not a string"))?)?);
                }
                Ok(cs)
            };
            let in_cols = cols_of("inflows", true)?;
            let out_cols = cols_of("outflows", false)?;
            for (k, rk) in res.iter_mut().enumerate().skip(1) {
                let (r0, rp) = (m.row(k), m.row(k - 1));
                let dl = (r0[level] - rp[level]) / dt;
                let qin: f64 = in_cols.iter().map(|&c| r0[c]).sum();
                let qout: f64 = out_cols.iter().map(|&c| r0[c]).sum();
                *rk = area * dl - factor * (qin - qout);
            }
        }
        other => return Err(format!("unknown balance type '{other}'")),
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataMatrix;

    /// A `mass_tank_volume` roles doc with area = dt = factor = 1, so the closure reduces to
    /// `residual[k] = (LIT[k] - LIT[k-1]) - (FIT_IN - FIT_OUT)` — easy to reason about by hand.
    fn tank_roles() -> RolesDoc {
        serde_json::from_str(
            r#"{"dataset":"t","balance":{"type":"mass_tank_volume","area_m2":1.0,"dt":1.0,
                 "level":"LIT","inflows":["FIT_IN"],"outflows":["FIT_OUT"],"flow_to_vol_per_dt":1.0}}"#,
        )
        .unwrap()
    }

    /// Build a 3-column tank matrix from a level series; inflow 10, outflow 8 (net +2) every step.
    fn tank_matrix(levels: &[f64]) -> DataMatrix {
        let rows: Vec<Vec<f64>> = levels.iter().map(|&l| vec![l, 10.0, 8.0]).collect();
        DataMatrix::new(vec!["LIT".into(), "FIT_IN".into(), "FIT_OUT".into()], rows)
    }

    #[test]
    fn balanced_tank_closes_to_zero() {
        // Level integrates the net flow exactly (+2/step) => the closure residual is ~0 everywhere.
        let r = balance_residual(
            &tank_matrix(&[100.0, 102.0, 104.0, 106.0, 108.0]),
            &tank_roles(),
        )
        .unwrap();
        for v in &r[1..] {
            assert!(v.abs() < 1e-9, "balanced closure must be ~0, got {v}");
        }
    }

    #[test]
    fn leaking_tank_breaks_the_closure() {
        // Level rises only +1/step while the metered net flow is +2 => an unmetered leak of 1/step =>
        // a sustained closure residual of -1 (the witness's evidence that the balance does not close).
        let r = balance_residual(
            &tank_matrix(&[100.0, 101.0, 102.0, 103.0, 104.0]),
            &tank_roles(),
        )
        .unwrap();
        for v in &r[1..] {
            assert!(
                (v - (-1.0)).abs() < 1e-9,
                "leak closure must be -1, got {v}"
            );
        }
    }

    #[test]
    fn unknown_balance_type_is_an_error() {
        let roles: RolesDoc =
            serde_json::from_str(r#"{"dataset":"t","balance":{"type":"nope"}}"#).unwrap();
        assert!(balance_residual(&tank_matrix(&[1.0, 2.0]), &roles).is_err());
    }
}
