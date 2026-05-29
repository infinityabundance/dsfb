//! `UnitConsistencyCourtV1` (Wave-3 physics) — a deterministic unit / dimension checker over residual
//! streams, so a **unit problem cannot masquerade as a process anomaly**.
//!
//! A large fraction of "anomalies" a naive monitor flags are not process events at all: someone wired a
//! °C tag and a K tag into the same balance, or compared a `bar` reading against a `Pa` reading, or treated
//! a mass fraction (`kg/kg`, 0..1) as a mole fraction (`mol/mol`) or as a `wt%` (0..100). Each of these
//! produces a large, persistent "residual" that is pure unit error. This court takes the **declared units**
//! of the channels that participate together in an expression (a balance term, a sensor-vs-prediction
//! comparison, a redundant-sensor pair) and deterministically proves they are dimensionally and
//! dimensionally-scaled compatible — emitting a typed hazard when they are not.
//!
//! Every quantity reduces to a vector of signed SI base-dimension exponents (mass, length, time,
//! temperature, amount); two channels combined in one expression must share that vector **and** their
//! multiplicative scale to SI, additive offset to SI (temperature is affine), and dimensionless *basis*
//! (mass-fraction ≠ mole-fraction even though both are dimensionless). The five outcomes —
//! `Consistent` / `DimensionMismatch` / `ScaleMismatch` / `AffineOffsetHazard` / `BasisMismatch` — name
//! exactly *why* an apparent residual is really a unit bug.
//!
//! Bounded (non-claims): this court checks the *declared* units only — it does not verify that a balance
//! equation is physically correct, that a sensor is calibrated, or that a unit label is truthful; a unit
//! string it cannot parse is reported as `UnknownUnit` (a fail), never silently assumed compatible.
//! Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Signed SI base-dimension exponents. Two quantities are dimensionally comparable iff these are equal.
/// Five bases cover chemical-process residuals: mass (kg), length (m), time (s), temperature (K),
/// amount of substance (mol). (Electrical/luminous bases are not needed here.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub mass: i8,
    pub length: i8,
    pub time: i8,
    pub temperature: i8,
    pub amount: i8,
}

impl Dimension {
    const fn new(mass: i8, length: i8, time: i8, temperature: i8, amount: i8) -> Self {
        Dimension {
            mass,
            length,
            time,
            temperature,
            amount,
        }
    }
    /// Compact `M^a L^b T^c Θ^d N^e` tag (omitting zero exponents); `"1"` when dimensionless.
    fn tag(self) -> String {
        let parts: [(&str, i8); 5] = [
            ("M", self.mass),
            ("L", self.length),
            ("T", self.time),
            ("Θ", self.temperature),
            ("N", self.amount),
        ];
        let mut s = String::new();
        for (sym, e) in parts {
            if e != 0 {
                s.push_str(sym);
                if e != 1 {
                    s.push('^');
                    s.push_str(&e.to_string());
                }
            }
        }
        if s.is_empty() {
            "1".into()
        } else {
            s
        }
    }
}

/// The dimensionless *basis* — two dimensionless quantities are only interchangeable if their basis matches.
/// A mass fraction and a mole fraction are both dimensionless yet not interconvertible without molar mass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Basis {
    /// A genuine pure number (ratio, efficiency, dimensionless group).
    Pure,
    /// kg/kg (or wt%) — a mass fraction.
    MassFraction,
    /// mol/mol (or mol%) — a mole fraction.
    MoleFraction,
}

impl Basis {
    fn tag(self) -> &'static str {
        match self {
            Basis::Pure => "pure",
            Basis::MassFraction => "mass_fraction",
            Basis::MoleFraction => "mole_fraction",
        }
    }
}

/// A parsed engineering unit: its dimension, the multiplicative `scale` and additive `offset` that take a
/// value to SI (`si = scale·value + offset`), and the dimensionless basis. Only temperature uses a nonzero
/// offset; everything else is purely multiplicative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    pub dimension: Dimension,
    pub scale: f64,
    pub offset: f64,
    pub basis: Basis,
}

// ── Dimension constants (named for the chemical-process quantities the court reasons about) ──────────
const TEMPERATURE: Dimension = Dimension::new(0, 0, 0, 1, 0);
const PRESSURE: Dimension = Dimension::new(1, -1, -2, 0, 0);
const MASS_FLOW: Dimension = Dimension::new(1, 0, -1, 0, 0);
const MOLAR_FLOW: Dimension = Dimension::new(0, 0, -1, 0, 1);
const VOLUME_FLOW: Dimension = Dimension::new(0, 3, -1, 0, 0);
const RATE: Dimension = Dimension::new(0, 0, -1, 0, 0);
const LENGTH: Dimension = Dimension::new(0, 1, 0, 0, 0);
const VOLUME: Dimension = Dimension::new(0, 3, 0, 0, 0);
const MASS: Dimension = Dimension::new(1, 0, 0, 0, 0);
const POWER: Dimension = Dimension::new(1, 2, -3, 0, 0);
const DIMENSIONLESS: Dimension = Dimension::new(0, 0, 0, 0, 0);

/// Parse an engineering-unit string into a [`Unit`]. Case-insensitive; trims whitespace; accepts the common
/// ASCII and `°` spellings. Returns `None` for an unrecognised unit (the court reports that as a fail rather
/// than guessing). The table is deliberately broad — it is prior-art disclosure of the unit surface DSFB
/// reasons over, not a minimal set.
pub fn parse_unit(s: &str) -> Option<Unit> {
    let k = s.trim().to_lowercase().replace('^', ""); // accept "m^3/h" / "cm^3/s" spellings
    let k = k.trim_start_matches('°'); // accept "°C" / "°F"
    let u = |dimension, scale, offset, basis| {
        Some(Unit {
            dimension,
            scale,
            offset,
            basis,
        })
    };
    match k {
        // Temperature (affine: si_K = scale·v + offset).
        "k" | "kelvin" => u(TEMPERATURE, 1.0, 0.0, Basis::Pure),
        "c" | "degc" | "celsius" => u(TEMPERATURE, 1.0, 273.15, Basis::Pure),
        "f" | "degf" | "fahrenheit" => u(
            TEMPERATURE,
            5.0 / 9.0,
            273.15 - 32.0 * 5.0 / 9.0,
            Basis::Pure,
        ),
        // Pressure (to Pa).
        "pa" => u(PRESSURE, 1.0, 0.0, Basis::Pure),
        "kpa" => u(PRESSURE, 1e3, 0.0, Basis::Pure),
        "mbar" => u(PRESSURE, 1e2, 0.0, Basis::Pure),
        "bar" => u(PRESSURE, 1e5, 0.0, Basis::Pure),
        "psi" | "psig" => u(PRESSURE, 6894.757, 0.0, Basis::Pure),
        "atm" => u(PRESSURE, 101_325.0, 0.0, Basis::Pure),
        // Mass flow (to kg/s).
        "kg/s" => u(MASS_FLOW, 1.0, 0.0, Basis::Pure),
        "kg/min" => u(MASS_FLOW, 1.0 / 60.0, 0.0, Basis::Pure),
        "kg/h" | "kg/hr" => u(MASS_FLOW, 1.0 / 3600.0, 0.0, Basis::Pure),
        "t/h" | "tonne/h" => u(MASS_FLOW, 1000.0 / 3600.0, 0.0, Basis::Pure),
        "g/s" => u(MASS_FLOW, 1e-3, 0.0, Basis::Pure),
        // Molar flow (to mol/s).
        "mol/s" => u(MOLAR_FLOW, 1.0, 0.0, Basis::Pure),
        "mol/min" => u(MOLAR_FLOW, 1.0 / 60.0, 0.0, Basis::Pure),
        "mol/h" => u(MOLAR_FLOW, 1.0 / 3600.0, 0.0, Basis::Pure),
        "kmol/h" => u(MOLAR_FLOW, 1000.0 / 3600.0, 0.0, Basis::Pure),
        // Volumetric flow (to m^3/s).
        "m3/s" => u(VOLUME_FLOW, 1.0, 0.0, Basis::Pure),
        "m3/h" => u(VOLUME_FLOW, 1.0 / 3600.0, 0.0, Basis::Pure),
        "l/s" => u(VOLUME_FLOW, 1e-3, 0.0, Basis::Pure),
        "l/min" | "lpm" => u(VOLUME_FLOW, 1e-3 / 60.0, 0.0, Basis::Pure),
        "l/h" | "lph" => u(VOLUME_FLOW, 1e-3 / 3600.0, 0.0, Basis::Pure),
        "gpm" => u(VOLUME_FLOW, 6.309_02e-5, 0.0, Basis::Pure), // US gallon/min
        // Rate / frequency (to 1/s).
        "1/s" | "hz" | "s-1" => u(RATE, 1.0, 0.0, Basis::Pure),
        "1/min" => u(RATE, 1.0 / 60.0, 0.0, Basis::Pure),
        "1/h" => u(RATE, 1.0 / 3600.0, 0.0, Basis::Pure),
        // Length (to m).
        "m" => u(LENGTH, 1.0, 0.0, Basis::Pure),
        "cm" => u(LENGTH, 1e-2, 0.0, Basis::Pure),
        "mm" => u(LENGTH, 1e-3, 0.0, Basis::Pure),
        // Volume (to m^3).
        "m3" => u(VOLUME, 1.0, 0.0, Basis::Pure),
        "l" | "liter" | "litre" => u(VOLUME, 1e-3, 0.0, Basis::Pure),
        // Mass (to kg).
        "kg" => u(MASS, 1.0, 0.0, Basis::Pure),
        "g" => u(MASS, 1e-3, 0.0, Basis::Pure),
        // Power / heat duty (to W).
        "w" => u(POWER, 1.0, 0.0, Basis::Pure),
        "kw" => u(POWER, 1e3, 0.0, Basis::Pure),
        "j/s" => u(POWER, 1.0, 0.0, Basis::Pure),
        "j/min" => u(POWER, 1.0 / 60.0, 0.0, Basis::Pure),
        "kj/min" => u(POWER, 1000.0 / 60.0, 0.0, Basis::Pure),
        // Dimensionless and fractions (basis distinguishes mass vs mole; canonical scale is the 0..1 form,
        // so the 0..100 percentage forms carry scale 0.01 — a wt% vs kg/kg mix is a real ScaleMismatch).
        "" | "-" | "dimensionless" | "ratio" => u(DIMENSIONLESS, 1.0, 0.0, Basis::Pure),
        "%" => u(DIMENSIONLESS, 1e-2, 0.0, Basis::Pure),
        "kg/kg" | "mass_frac" | "massfraction" => u(DIMENSIONLESS, 1.0, 0.0, Basis::MassFraction),
        "wt%" | "wt_pct" | "mass%" => u(DIMENSIONLESS, 1e-2, 0.0, Basis::MassFraction),
        "mol/mol" | "mole_frac" | "molefraction" => u(DIMENSIONLESS, 1.0, 0.0, Basis::MoleFraction),
        "mol%" | "mole%" => u(DIMENSIONLESS, 1e-2, 0.0, Basis::MoleFraction),
        _ => None,
    }
}

/// True iff two SI scalars agree to a small relative tolerance (so `6894.757` parsed twice, or `1.0/3.0`
/// computed two ways, compares equal without false ScaleMismatch on float last-bit noise).
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()))
}

/// How two channels are combined — only flavours the human explanation; the rule is the same for both
/// (operands of one expression must be in identical units, because even a difference like `T_out − T_in`
/// is only valid when both legs carry the *same* unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitRelation {
    /// The channels are claimed to be the same physical quantity (redundant sensors; measurement vs prediction).
    SameQuantity,
    /// The channels are added / subtracted together in a balance term.
    AdditivelyCombined,
}

impl UnitRelation {
    fn tag(self) -> &'static str {
        match self {
            UnitRelation::SameQuantity => "same_quantity",
            UnitRelation::AdditivelyCombined => "additively_combined",
        }
    }
}

/// One claim that two channels combine in a single expression and must therefore share units.
#[derive(Debug, Clone)]
pub struct UnitAssertion {
    /// Where the combination occurs (e.g. `"three-tank closure: accumulation vs inter-tank flow"`).
    pub context: String,
    pub a_channel: String,
    pub a_unit: String,
    pub b_channel: String,
    pub b_unit: String,
    pub relation: UnitRelation,
}

/// The typed outcome of checking one assertion — names *why* an apparent residual is really a unit bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitHazard {
    /// The two units are identical to SI (dimension, scale, offset, basis) — the combination is valid.
    Consistent,
    /// A unit string could not be parsed — reported, never silently assumed compatible.
    UnknownUnit,
    /// Different base dimensions (e.g. pressure vs temperature) — the combination is meaningless.
    DimensionMismatch,
    /// Same dimension, different multiplicative scale (e.g. `bar` vs `Pa`, factor 1e5; `wt%` vs `kg/kg`,
    /// factor 100) — a raw comparison or sum is wrong by that ratio.
    ScaleMismatch,
    /// Same dimension and scale, different additive offset (e.g. `°C` vs `K`) — the affine origin differs,
    /// so a direct comparison sees a constant ~273 "residual" that is pure unit error.
    AffineOffsetHazard,
    /// Both dimensionless but a different basis (mass fraction vs mole fraction) — not interconvertible
    /// without molar mass; treating them as one quantity is a basis error.
    BasisMismatch,
}

impl UnitHazard {
    fn tag(self) -> &'static str {
        match self {
            UnitHazard::Consistent => "consistent",
            UnitHazard::UnknownUnit => "unknown_unit",
            UnitHazard::DimensionMismatch => "dimension_mismatch",
            UnitHazard::ScaleMismatch => "scale_mismatch",
            UnitHazard::AffineOffsetHazard => "affine_offset_hazard",
            UnitHazard::BasisMismatch => "basis_mismatch",
        }
    }
    /// True iff the combination is valid (no hazard).
    pub fn is_consistent(self) -> bool {
        matches!(self, UnitHazard::Consistent)
    }
}

/// Classify one assertion's two units into a [`UnitHazard`]. Checked in physical priority order: an
/// unparseable unit first, then dimension, then dimensionless basis, then scale, then affine offset —
/// so the *most fundamental* discrepancy is the one reported.
fn classify(a_unit: &str, b_unit: &str) -> UnitHazard {
    let (a, b) = match (parse_unit(a_unit), parse_unit(b_unit)) {
        (Some(a), Some(b)) => (a, b),
        _ => return UnitHazard::UnknownUnit,
    };
    if a.dimension != b.dimension {
        return UnitHazard::DimensionMismatch;
    }
    if a.basis != b.basis {
        return UnitHazard::BasisMismatch;
    }
    if !near(a.scale, b.scale) {
        return UnitHazard::ScaleMismatch;
    }
    if !near(a.offset, b.offset) {
        return UnitHazard::AffineOffsetHazard;
    }
    UnitHazard::Consistent
}

/// One checked assertion: the context, the two `channel [unit]` legs, the relation, the hazard, and a
/// human explanation citing the SI dimension / scale ratio so the verdict is auditable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitFinding {
    pub context: String,
    pub a: String,
    pub b: String,
    pub relation: String,
    pub hazard: String,
    pub explanation: String,
}

/// A hash-sealed unit-consistency verdict (schema v1) over a set of unit assertions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitConsistencyCourtV1 {
    pub assertions_checked: usize,
    pub findings: Vec<UnitFinding>,
    /// Count of `Consistent` findings.
    pub n_consistent: usize,
    /// Count of hazard findings (anything not `Consistent`).
    pub n_hazard: usize,
    pub court_hash: String,
}

impl UnitConsistencyCourtV1 {
    fn explain(hazard: UnitHazard, a_unit: &str, b_unit: &str, relation: UnitRelation) -> String {
        let combined = match relation {
            UnitRelation::SameQuantity => "compared as the same quantity",
            UnitRelation::AdditivelyCombined => "added/subtracted in a balance term",
        };
        match hazard {
            UnitHazard::Consistent => {
                let d = parse_unit(a_unit)
                    .map(|u| u.dimension.tag())
                    .unwrap_or_else(|| "?".into());
                format!("identical units [{a_unit}] (dimension {d}); {combined} is valid")
            }
            UnitHazard::UnknownUnit => {
                let bad = if parse_unit(a_unit).is_none() {
                    a_unit
                } else {
                    b_unit
                };
                format!("unit '{bad}' is not recognised; refusing to assume compatibility")
            }
            UnitHazard::DimensionMismatch => {
                let da = parse_unit(a_unit)
                    .map(|u| u.dimension.tag())
                    .unwrap_or_default();
                let db = parse_unit(b_unit)
                    .map(|u| u.dimension.tag())
                    .unwrap_or_default();
                format!("different dimensions: [{a_unit}]={da} vs [{b_unit}]={db}; {combined} is meaningless")
            }
            UnitHazard::ScaleMismatch => {
                let ratio = match (parse_unit(a_unit), parse_unit(b_unit)) {
                    (Some(a), Some(b)) if b.scale != 0.0 => a.scale / b.scale,
                    _ => f64::NAN,
                };
                format!(
                    "same dimension, scale differs by ×{ratio:.6}: [{a_unit}] vs [{b_unit}]; \
                         {combined} without conversion is wrong by that factor"
                )
            }
            UnitHazard::AffineOffsetHazard => format!(
                "affine offset differs: [{a_unit}] vs [{b_unit}] (e.g. °C vs K); {combined} sees a \
                 constant ~273 offset that is pure unit error, not a process residual"
            ),
            UnitHazard::BasisMismatch => {
                let ba = parse_unit(a_unit).map(|u| u.basis.tag()).unwrap_or("?");
                let bb = parse_unit(b_unit).map(|u| u.basis.tag()).unwrap_or("?");
                format!(
                    "both dimensionless but different basis: [{a_unit}]={ba} vs [{b_unit}]={bb}; \
                     not interconvertible without molar mass"
                )
            }
        }
    }

    fn seal(
        assertions_checked: usize,
        findings: &[UnitFinding],
        n_consistent: usize,
        n_hazard: usize,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"unit_consistency_court_v1");
        h.u64("assertions_checked", assertions_checked as u64);
        for f in findings {
            h.field("context", f.context.as_bytes());
            h.field("a", f.a.as_bytes());
            h.field("b", f.b.as_bytes());
            h.field("relation", f.relation.as_bytes());
            h.field("hazard", f.hazard.as_bytes());
        }
        h.u64("n_consistent", n_consistent as u64);
        h.u64("n_hazard", n_hazard as u64);
        h.finalize_hex()
    }

    /// Check every assertion and seal the verdict.
    pub fn build(assertions: &[UnitAssertion]) -> Self {
        let mut findings = Vec::with_capacity(assertions.len());
        for a in assertions {
            let hazard = classify(&a.a_unit, &a.b_unit);
            findings.push(UnitFinding {
                context: a.context.clone(),
                a: format!("{} [{}]", a.a_channel, a.a_unit),
                b: format!("{} [{}]", a.b_channel, a.b_unit),
                relation: a.relation.tag().to_string(),
                hazard: hazard.tag().to_string(),
                explanation: Self::explain(hazard, &a.a_unit, &a.b_unit, a.relation),
            });
        }
        let n_consistent = findings.iter().filter(|f| f.hazard == "consistent").count();
        let n_hazard = findings.len() - n_consistent;
        let court_hash = Self::seal(assertions.len(), &findings, n_consistent, n_hazard);
        UnitConsistencyCourtV1 {
            assertions_checked: assertions.len(),
            findings,
            n_consistent,
            n_hazard,
            court_hash,
        }
    }

    /// All assertions are unit-consistent iff no hazard was found.
    pub fn all_consistent(&self) -> bool {
        self.n_hazard == 0
    }

    /// Re-derive the tallies and seal from the stored findings and compare the whole record — catches
    /// tampering of a hazard label, a tally, or the hash (the tallies are a pure function of the findings).
    pub fn verify(&self) -> bool {
        let n_consistent = self
            .findings
            .iter()
            .filter(|f| f.hazard == "consistent")
            .count();
        let n_hazard = self.findings.len() - n_consistent;
        n_consistent == self.n_consistent
            && n_hazard == self.n_hazard
            && Self::seal(
                self.assertions_checked,
                &self.findings,
                n_consistent,
                n_hazard,
            ) == self.court_hash
    }

    /// Plain-text render (one line per finding + a verdict), for the CLI and the report file.
    pub fn render(&self) -> String {
        let mut s = String::new();
        for f in &self.findings {
            let mark = if f.hazard == "consistent" {
                "  OK   "
            } else {
                "  HAZARD "
            };
            s.push_str(mark);
            s.push_str(&f.context);
            s.push_str(&format!(" — {} vs {}: {}\n", f.a, f.b, f.explanation));
        }
        s.push_str(&format!(
            "verdict: {} ({} consistent, {} hazard)\ncourt_hash: {}\n",
            if self.all_consistent() {
                "UNIT-CONSISTENT"
            } else {
                "UNIT-HAZARD"
            },
            self.n_consistent,
            self.n_hazard,
            self.court_hash
        ));
        s
    }
}

/// Derive the unit-consistency assertions implied by a documented balance: the **groups of channels the
/// balance adds or subtracts**, which must therefore carry identical units. Quantities combined only
/// through a coefficient (a level reconciled to a flow via tank area, a temperature scaled by `ρcₚV`) are
/// deliberately *different* dimensions and are NOT asserted equal — only the directly summed/subtracted
/// legs are. Channels absent from the roles' variable list are skipped (a missing-unit data issue, not a
/// unit hazard); a channel whose unit string the parser cannot read still surfaces as `UnknownUnit`.
///
/// This is the court applied to DSFB's *own* documented balances: a clean verdict means every balance the
/// framework computes combines like-united quantities; a hazard would mean a balance was misconfigured
/// (e.g. one tank level in `cm` and another in `m`) — caught before it reads out as a spurious residual.
pub fn assertions_for_balance(roles: &crate::balance::RolesDoc) -> Vec<UnitAssertion> {
    let b = &roles.balance;
    let btype = b.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let unit_of = |name: &str| -> Option<String> {
        roles
            .variables
            .iter()
            .find(|v| v.name == name)
            .map(|v| v.unit.clone())
    };
    // The actual column name behind a balance role key (e.g. `balance.reactor_temp` → "reactor_temp").
    let named =
        |key: &str| -> Option<String> { b.get(key).and_then(|v| v.as_str()).map(str::to_string) };
    // The string members of a balance array key (e.g. `balance.levels` / `balance.inflows`).
    let array = |key: &str| -> Vec<String> {
        b.get(key)
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    // Emit pairwise assertions tying every member of a same-unit group to its first member (transitive).
    let mut group = |ctx: &str, members: &[String], rel: UnitRelation| {
        let mut named_members: Vec<(String, String)> = members
            .iter()
            .filter_map(|m| unit_of(m).map(|u| (m.clone(), u)))
            .collect();
        if named_members.len() < 2 {
            return;
        }
        let (head_ch, head_u) = named_members.remove(0);
        for (ch, u) in named_members {
            out.push(UnitAssertion {
                context: ctx.to_string(),
                a_channel: head_ch.clone(),
                a_unit: head_u.clone(),
                b_channel: ch,
                b_unit: u,
                relation: rel,
            });
        }
    };
    match btype {
        "mass_three_tank" => group(
            "three-tank: inter-tank level differences",
            &array("levels"),
            UnitRelation::AdditivelyCombined,
        ),
        "mass_quad_tank" => {
            // Tank-1 balance subtracts the tank-1 and tank-3 outflow terms; both levels must share a unit.
            let levels: Vec<String> = ["level_1", "level_3"]
                .iter()
                .filter_map(|k| named(k))
                .collect();
            group(
                "quad-tank: tank-1 vs tank-3 level terms",
                &levels,
                UnitRelation::AdditivelyCombined,
            );
        }
        "energy_cstr" => {
            let temps: Vec<String> = [
                "reactor_temp",
                "feed_temp",
                "coolant_temp_in",
                "coolant_temp_out",
            ]
            .iter()
            .filter_map(|k| named(k))
            .collect();
            group(
                "CSTR energy: all temperatures in (Tin−T) / (Tc_out−Tc_in)",
                &temps,
                UnitRelation::AdditivelyCombined,
            );
        }
        "energy_csth" => {
            let temps: Vec<String> = ["tank_temp", "temp_cold_in"]
                .iter()
                .filter_map(|k| named(k))
                .collect();
            group(
                "CSTH energy: tank vs cold-inlet temperature in (Tc−T)",
                &temps,
                UnitRelation::AdditivelyCombined,
            );
        }
        "mass_tank_volume" => {
            let inflows = array("inflows");
            let outflows = array("outflows");
            group(
                "tank-volume: summed inflows",
                &inflows,
                UnitRelation::AdditivelyCombined,
            );
            group(
                "tank-volume: summed outflows",
                &outflows,
                UnitRelation::AdditivelyCombined,
            );
            // Inflows are subtracted from outflows (qin − qout); the two legs must share a unit too.
            if let (Some(i0), Some(o0)) = (inflows.first(), outflows.first()) {
                if let (Some(iu), Some(ou)) = (unit_of(i0), unit_of(o0)) {
                    out.push(UnitAssertion {
                        context: "tank-volume: inflow vs outflow (qin − qout)".into(),
                        a_channel: i0.clone(),
                        a_unit: iu,
                        b_channel: o0.clone(),
                        b_unit: ou,
                        relation: UnitRelation::AdditivelyCombined,
                    });
                }
            }
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(
        context: &str,
        ac: &str,
        au: &str,
        bc: &str,
        bu: &str,
        rel: UnitRelation,
    ) -> UnitAssertion {
        UnitAssertion {
            context: context.into(),
            a_channel: ac.into(),
            a_unit: au.into(),
            b_channel: bc.into(),
            b_unit: bu.into(),
            relation: rel,
        }
    }

    #[test]
    fn parse_covers_affine_pressure_and_fraction_units() {
        // Temperature is affine: °C and K share dimension+scale but differ by the 273.15 offset.
        let c = parse_unit("degC").unwrap();
        let k = parse_unit("K").unwrap();
        assert_eq!(c.dimension, k.dimension);
        assert!(near(c.scale, k.scale) && !near(c.offset, k.offset));
        // bar→Pa is a pure 1e5 scale; mass- and mole-fraction are dimensionless with distinct bases.
        assert!(near(parse_unit("bar").unwrap().scale, 1e5));
        assert_eq!(parse_unit("kg/kg").unwrap().basis, Basis::MassFraction);
        assert_eq!(parse_unit("mol/mol").unwrap().basis, Basis::MoleFraction);
        assert!(parse_unit("furlongs/fortnight").is_none());
    }

    #[test]
    fn classifies_each_hazard_class() {
        assert_eq!(classify("K", "K"), UnitHazard::Consistent);
        assert_eq!(classify("degC", "K"), UnitHazard::AffineOffsetHazard);
        assert_eq!(classify("bar", "Pa"), UnitHazard::ScaleMismatch);
        assert_eq!(classify("wt%", "kg/kg"), UnitHazard::ScaleMismatch); // factor 100, same basis
        assert_eq!(classify("kg/kg", "mol/mol"), UnitHazard::BasisMismatch);
        assert_eq!(classify("Pa", "K"), UnitHazard::DimensionMismatch);
        assert_eq!(classify("kg/s", "mol/s"), UnitHazard::DimensionMismatch); // mass flow ≠ molar flow
        assert_eq!(classify("nonsense", "K"), UnitHazard::UnknownUnit);
    }

    #[test]
    fn court_flags_a_mixed_unit_balance_and_self_verifies() {
        // A plausible misconfigured balance: a coolant-out temperature in °C compared against a reactor
        // temperature in K (offset hazard), and a feed flow in kg/h summed with one in kg/s (scale).
        let asserts = vec![
            a(
                "reactor energy: T_out vs T_reactor",
                "Tco",
                "degC",
                "T",
                "K",
                UnitRelation::SameQuantity,
            ),
            a(
                "reactor mass: feed vs recycle",
                "F_feed",
                "kg/h",
                "F_recycle",
                "kg/s",
                UnitRelation::AdditivelyCombined,
            ),
            a(
                "reactor mass: feed vs product",
                "F_feed",
                "kg/h",
                "F_prod",
                "kg/h",
                UnitRelation::AdditivelyCombined,
            ),
        ];
        let court = UnitConsistencyCourtV1::build(&asserts);
        assert_eq!(court.assertions_checked, 3);
        assert_eq!(court.n_hazard, 2);
        assert_eq!(court.n_consistent, 1);
        assert!(!court.all_consistent());
        assert_eq!(court.court_hash.len(), 64);
        assert!(court.verify());
        // Determinism: an identical assertion set seals to the identical hash.
        assert_eq!(UnitConsistencyCourtV1::build(&asserts), court);
    }

    #[test]
    fn clean_balance_is_unit_consistent() {
        let asserts = vec![
            a(
                "three-tank: levels share a length unit",
                "level_1",
                "cm",
                "level_2",
                "cm",
                UnitRelation::AdditivelyCombined,
            ),
            a(
                "tank volume: inflow vs outflow",
                "FIT_IN",
                "m3/h",
                "FIT_OUT",
                "m3/h",
                UnitRelation::AdditivelyCombined,
            ),
        ];
        let court = UnitConsistencyCourtV1::build(&asserts);
        assert!(court.all_consistent() && court.verify());
        assert_eq!(court.n_hazard, 0);
    }

    #[test]
    fn tampering_a_hazard_label_breaks_the_seal() {
        let asserts = vec![a("x", "p", "bar", "q", "Pa", UnitRelation::SameQuantity)];
        let mut court = UnitConsistencyCourtV1::build(&asserts);
        assert!(court.verify());
        // Forge the hazard to "consistent" to hide the scale bug — the tally re-derivation + seal catch it.
        court.findings[0].hazard = "consistent".into();
        assert!(!court.verify());
    }
}
