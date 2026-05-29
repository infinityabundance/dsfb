#!/usr/bin/env python3
"""Generate *instrumented* demonstrator slices that encode the metadata the 20 anonymised public
slices lack: variable roles (measured vs manipulated), engineering units, and an explicit
mass/energy-balance structure. These are the substrate for the mass/energy-balance *witness*
detectors and control-action contextualisation (which need to know which column is a flow, a
temperature, a manipulated variable, etc.).

Two kinds of demonstrator are emitted, all to crates/dsfb-chemical-engineering-edge/data/instrumented/.

A. BALANCE-WITNESS demonstrators, each with a JSON sidecar (`*.roles.json`) declaring roles/units and
   the balance equation, so a witness can recompute the closure from the raw columns:

  three_tank_instrumented     -- DTS200-style three-tank. MASS balance A dh/dt = q_in - q_out.
                                 Fault: a LEAK in tank 2 (unmeasured outflow) breaking the closure.
  cstr_instrumented           -- non-isothermal CSTR (Arrhenius). ENERGY balance. Fault: reactor-
                                 thermocouple DRIFT makes the measured temperature inconsistent.
  quadruple_tank_instrumented -- Johansson quadruple-tank. MASS balance. Fault: leak in tank 1.
  csth_instrumented           -- continuous stirred-tank heater. ENERGY balance. Fault: insulation loss.

  The script validates each balance-closure residual is stable under nominal operation and shifts
  markedly after the fault onset -- the witness detects the sustained shift relative to its calibrated
  baseline (not absolute zero-closure: a real balance carries a model/measurement offset).

B. CONTROL-LOOP fault demonstrators (F1 / F9), with NO roles sidecar -- they are not balance witnesses
   but exhibit the fault's residual motif on the cheap PV/OP sensors, detected by the standard atlas
   detector bank (spectral-entropy + Page-Hinkley on the SPE residual). The "executed" evidence is the
   committed CSV + the gated test `tests/fault_demonstrators.rs`:

  valve_stiction_instrumented -- F1: a near-static valve under a benign OP dither; after onset the stem
                                 STICKS (moves only when |OP-stem|>S, then slips) -> stick-slip limit cycle.
  valve_hunting_instrumented  -- F9: same loop; after onset the positioner HUNTS (stem = OP + fixed-period
                                 oscillation) -> a sustained limit-cycle with a dominant spectral peak.

Deterministic (seeded noise), no network.

Run:  python3 scripts/gen_instrumented.py
"""
import csv
import json
import math
import os

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(HERE, "data", "instrumented")
os.makedirs(OUT, exist_ok=True)


class Rng:
    """SplitMix64 -> uniform/normal. Mirrors datasets.rs::Rng so the demonstrators are deterministic
    and independent of NumPy. Seeded per dataset."""

    def __init__(self, seed):
        self.s = seed & 0xFFFFFFFFFFFFFFFF

    def _next(self):
        self.s = (self.s + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
        z = self.s
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
        return (z ^ (z >> 31)) & 0xFFFFFFFFFFFFFFFF

    def uniform(self):
        return (self._next() >> 11) / float(1 << 53)

    def normal(self, sd):
        # Box-Muller; clamp to +/-5 sigma like the Rust generator.
        u1 = max(self.uniform(), 1e-12)
        u2 = self.uniform()
        z = math.sqrt(-2.0 * math.log(u1)) * math.cos(2.0 * math.pi * u2)
        return max(-5.0, min(5.0, z)) * sd


# ──────────────────────────────────────────────────────────────────────────────
# Three-tank: mass-balance witness (leak fault)
# ──────────────────────────────────────────────────────────────────────────────
def gen_three_tank(n=720, onset=420, seed=11):
    """DTS200-style three tanks in series. State = levels h1,h2,h3 [cm]. Inlet pump q_in [cm^3/s]
    is the manipulated variable (PI control holding h1 at a setpoint). Inter-tank and outlet flows
    follow Torricelli q = c*sgn(dh)*sqrt(|dh|). At `onset` a leak opens in tank 2 (q_leak), an
    *unmeasured* outflow that breaks the tank-2 mass balance A dh2/dt = q12 - q23.

    Columns (measured unless noted): level_1, level_2, level_3 [cm]; q_in [cm^3/s, MANIPULATED];
    label (0/1 leak), phase (0 fill / 1 steady).
    """
    A = 154.0       # tank cross-section [cm^2]
    c12, c23, c3 = 5.0, 5.0, 5.0    # Torricelli valve coefficients [cm^2.5/s]
    dt = 1.0        # s
    sp = 40.0       # h1 setpoint [cm]
    kp, ki = 8.0, 0.05
    q_leak_coeff = 10.0  # leak valve coefficient (tank 2), engaged after onset
    rng = Rng(seed)
    h1 = h2 = h3 = 15.0
    integ = 0.0
    rows = []
    tor = lambda c, dh: c * math.copysign(math.sqrt(abs(dh)), dh)
    for k in range(n):
        # PI level controller on tank 1 -> manipulated inflow q_in (saturated to a pump range).
        err = sp - h1
        integ += err * dt
        q_in = max(0.0, min(120.0, 60.0 + kp * err + ki * integ))
        q12 = tor(c12, h1 - h2)
        q23 = tor(c23, h2 - h3)
        q_out = c3 * math.sqrt(max(h3, 0.0))
        leak = q_leak_coeff * math.sqrt(max(h2, 0.0)) if k >= onset else 0.0
        # True mass balances (Euler step).
        h1 += dt * (q_in - q12) / A
        h2 += dt * (q12 - q23 - leak) / A
        h3 += dt * (q23 - q_out) / A
        h1, h2, h3 = max(h1, 0.0), max(h2, 0.0), max(h3, 0.0)
        rows.append({
            "level_1": h1 + rng.normal(0.02),
            "level_2": h2 + rng.normal(0.02),
            "level_3": h3 + rng.normal(0.02),
            "q_in": q_in + rng.normal(0.2),
            "label": 1 if k >= onset else 0,
            "phase": 0 if k < 120 else 1,   # 0 = fill-up regime, 1 = steady operation
        })
    roles = {
        "dataset": "three_tank_instrumented",
        "kind": "simulation",
        "description": "DTS200-style three-tank; mass-balance witness; leak fault in tank 2.",
        "variables": [
            {"name": "level_1", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "level_2", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "level_3", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "q_in", "role": "manipulated", "unit": "cm^3/s", "quantity": "inflow"},
        ],
        # Mass-balance witness on tank 2: A*dh2/dt should equal (q12 - q23); the residual is the
        # unaccounted flow (the leak). Torricelli coefficients let the witness reconstruct q12,q23.
        "balance": {
            "type": "mass_three_tank",
            "tank_area_cm2": A,
            "c12": c12, "c23": c23, "c3": c3, "dt_s": dt,
            "levels": ["level_1", "level_2", "level_3"],
            "witness_tank": 2,
            "definition": "residual = A*dh2/dt - (c12*sgn(h1-h2)sqrt|h1-h2| - c23*sgn(h2-h3)sqrt|h2-h3|)",
        },
        "fault": {"onset_index": onset, "mode": "unmeasured leak in tank 2"},
    }
    return rows, roles


def three_tank_closure(rows, roles):
    """Validation: recompute the tank-2 mass-balance closure residual from the *measured* columns
    exactly as the witness will, and return (mean|res| nominal, mean|res| fault)."""
    b = roles["balance"]
    A, c12, c23, dt = b["tank_area_cm2"], b["c12"], b["c23"], b["dt_s"]
    onset = roles["fault"]["onset_index"]
    tor = lambda c, dh: c * math.copysign(math.sqrt(abs(dh)), dh)
    nom, flt = [], []
    for k in range(1, len(rows)):
        h1, h2, h3 = rows[k]["level_1"], rows[k]["level_2"], rows[k]["level_3"]
        dh2 = (rows[k]["level_2"] - rows[k - 1]["level_2"]) / dt
        res = A * dh2 - (tor(c12, h1 - h2) - tor(c23, h2 - h3))
        (flt if k >= onset else nom).append(abs(res))
    return sum(nom) / len(nom), sum(flt) / len(flt)


# ──────────────────────────────────────────────────────────────────────────────
# CSTR: energy-balance witness + control-action (thermocouple drift fault)
# ──────────────────────────────────────────────────────────────────────────────
def gen_cstr(n=720, onset=420, seed=23):
    """Non-isothermal CSTR with Arrhenius kinetics. Manipulated: coolant_flow (PI control holding
    reactor_temp at setpoint; saturates as it fights the fault). At `onset` the reactor thermocouple
    develops a slow DRIFT, so the *reported* reactor_temp becomes inconsistent with the energy flows
    and the energy-balance closure (computed from reported signals) breaks while the true plant is
    fine -- the classic "is it the process or the sensor?" question the witness answers.

    Columns: feed_flow [L/min], feed_temp [K], feed_conc [mol/L], reactor_temp [K, reported],
    reactor_conc [mol/L], coolant_flow [L/min, MANIPULATED], coolant_temp_in [K],
    coolant_temp_out [K]; label, phase.
    """
    V = 100.0          # L
    rho_cp = 0.239 * 1000.0   # J/(L*K) (approx, lumped)
    dHr = -5.0e4       # J/mol (exothermic)
    k0, EoverR = 7.2e10, 8750.0
    F, Ca0, Tin = 10.0, 1.0, 350.0
    rho_cp_c = 4.18 * 1000.0  # coolant J/(L*K)
    Tc_in = 300.0
    sp = 350.0         # reactor temp setpoint [K]
    kp, ki = 1.5, 0.02
    dt = 0.5           # min (scaled for a visible trajectory)
    rng = Rng(seed)
    Ca, T = 0.5, 350.0
    integ = 0.0
    rows = []
    for k in range(n):
        r = k0 * math.exp(-EoverR / T) * Ca   # reaction rate [mol/(L*min)]
        # PI controller on reactor temp -> coolant flow (manipulated), saturated to a pump range.
        err = T - sp
        integ += err * dt
        Fc = max(2.0, min(120.0, 12.0 + kp * err + ki * integ))
        UA = 5.0e4
        # Coolant outlet temp from a lumped cooling-jacket energy balance (measured).
        Q_cool = UA * (T - Tc_in) * (Fc / (Fc + 20.0))   # heat removed [J/min]
        Tc_out = Tc_in + Q_cool / max(Fc * rho_cp_c, 1e-6)
        # True state update.
        dCa = F / V * (Ca0 - Ca) - r
        dT = F / V * (Tin - T) + (-dHr) * r / rho_cp - Q_cool / (rho_cp * V)
        Ca = max(Ca + dt * dCa, 0.0)
        T = T + dt * dT
        # Thermocouple drift fault: reported temp diverges from true T after onset.
        drift = 0.14 * (k - onset) if k >= onset else 0.0
        T_reported = T + drift
        rows.append({
            "feed_flow": F + rng.normal(0.02),
            "feed_temp": Tin + rng.normal(0.1),
            "feed_conc": Ca0 + rng.normal(0.005),
            "reactor_temp": T_reported + rng.normal(0.1),
            "reactor_conc": Ca + rng.normal(0.004),
            "coolant_flow": Fc + rng.normal(0.05),
            "coolant_temp_in": Tc_in + rng.normal(0.1),
            "coolant_temp_out": Tc_out + rng.normal(0.1),
            "label": 1 if k >= onset else 0,
            "phase": 0 if k < 120 else 1,
        })
    roles = {
        "dataset": "cstr_instrumented",
        "kind": "simulation",
        "description": "Non-isothermal CSTR; energy-balance witness + control-action; thermocouple drift fault.",
        "variables": [
            {"name": "feed_flow", "role": "measured", "unit": "L/min", "quantity": "flow"},
            {"name": "feed_temp", "role": "measured", "unit": "K", "quantity": "temperature"},
            {"name": "feed_conc", "role": "measured", "unit": "mol/L", "quantity": "concentration"},
            {"name": "reactor_temp", "role": "controlled", "unit": "K", "quantity": "temperature"},
            {"name": "reactor_conc", "role": "measured", "unit": "mol/L", "quantity": "concentration"},
            {"name": "coolant_flow", "role": "manipulated", "unit": "L/min", "quantity": "flow"},
            {"name": "coolant_temp_in", "role": "measured", "unit": "K", "quantity": "temperature"},
            {"name": "coolant_temp_out", "role": "measured", "unit": "K", "quantity": "temperature"},
        ],
        # Energy-balance witness: rho_cp*V*dT/dt - [F*rho_cp*(Tin-T) + (-dHr)*r*V - Q_cool] ~ 0.
        # Q_cool reconstructed from coolant flow and inlet/outlet temps (all measured).
        "balance": {
            "type": "energy_cstr",
            "V_L": V, "rho_cp_J_per_LK": rho_cp, "neg_dHr_J_per_mol": -dHr,
            "k0": k0, "EoverR_K": EoverR, "F_L_per_min": F, "rho_cp_coolant_J_per_LK": rho_cp_c,
            "dt_min": dt,
            "reactor_temp": "reactor_temp", "feed_temp": "feed_temp", "reactor_conc": "reactor_conc",
            "coolant_flow": "coolant_flow", "coolant_temp_in": "coolant_temp_in", "coolant_temp_out": "coolant_temp_out",
            "definition": "residual = rho_cp*V*dT/dt - (F*rho_cp*(Tin-T) + (-dHr)*k0*exp(-EoverR/T)*Ca*V - Fc*rho_cp_c*(Tc_out-Tc_in))",
        },
        "control_action": {"manipulated": "coolant_flow", "controlled": "reactor_temp", "saturation_high": 120.0},
        "fault": {"onset_index": onset, "mode": "reactor thermocouple drift"},
    }
    return rows, roles


def cstr_closure(rows, roles):
    b = roles["balance"]
    V, rho_cp, neg_dHr = b["V_L"], b["rho_cp_J_per_LK"], b["neg_dHr_J_per_mol"]
    k0, EoverR, F = b["k0"], b["EoverR_K"], b["F_L_per_min"]
    rho_cp_c, dt, Tin = b["rho_cp_coolant_J_per_LK"], b["dt_min"], rows[0]["feed_temp"]
    onset = roles["fault"]["onset_index"]
    nom, flt = [], []
    for k in range(1, len(rows)):
        T = rows[k]["reactor_temp"]
        Ca = rows[k]["reactor_conc"]
        dT = (rows[k]["reactor_temp"] - rows[k - 1]["reactor_temp"]) / dt
        Fc = rows[k]["coolant_flow"]
        Q_cool = Fc * rho_cp_c * (rows[k]["coolant_temp_out"] - rows[k]["coolant_temp_in"])
        rxn = neg_dHr * k0 * math.exp(-EoverR / T) * Ca * V
        res = rho_cp * V * dT - (F * rho_cp * (rows[k]["feed_temp"] - T) + rxn - Q_cool)
        (flt if k >= onset else nom).append(abs(res))
    return sum(nom) / len(nom), sum(flt) / len(flt)


# ──────────────────────────────────────────────────────────────────────────────
# Quadruple-tank (Johansson 2000): mass-balance witness (leak in tank 1)
# ──────────────────────────────────────────────────────────────────────────────
def gen_quadruple_tank(n=720, onset=420, seed=31):
    """Johansson (2000) quadruple-tank. Pump 1 (v1) feeds tank 1 (fraction g1) and tank 4 (1-g1);
    pump 2 (v2) feeds tank 2 (g2) and tank 3 (1-g2). Tanks 3,4 drain into tanks 1,2. Outflows follow
    Torricelli a_i*sqrt(2g h_i). MASS balance per tank; at `onset` an unmeasured LEAK opens in tank 1,
    breaking its balance A1 dh1/dt = a3 sqrt(2g h3) - a1 sqrt(2g h1) + g1 k1 v1.

    Columns: level_1..level_4 [cm]; v1,v2 [pump voltage V, MANIPULATED]; label, phase.
    """
    A = [28.0, 32.0, 28.0, 32.0]      # tank areas [cm^2]
    a = [0.071, 0.057, 0.071, 0.057]  # outlet areas [cm^2]
    g = 981.0                          # [cm/s^2]
    k1, k2 = 3.33, 3.35                # pump gains [cm^3/(V s)]
    g1, g2 = 0.70, 0.60                # valve split ratios
    dt = 1.0
    leak_coeff = 0.10
    rng = Rng(seed)
    h = [12.0, 12.0, 5.0, 5.0]
    i1 = i2 = 0.0
    rows = []
    sq = lambda x: math.sqrt(max(x, 0.0))
    for k in range(n):
        e1, e2 = 12.0 - h[0], 12.0 - h[1]   # PI level control to setpoint 12 cm
        i1 += e1 * dt
        i2 += e2 * dt
        v1 = max(0.0, min(12.0, 3.0 + 0.8 * e1 + 0.01 * i1))
        v2 = max(0.0, min(12.0, 3.0 + 0.8 * e2 + 0.01 * i2))
        q1, q2, q3, q4 = a[0] * sq(2 * g * h[0]), a[1] * sq(2 * g * h[1]), a[2] * sq(2 * g * h[2]), a[3] * sq(2 * g * h[3])
        leak = leak_coeff * sq(2 * g * h[0]) if k >= onset else 0.0
        h = [
            max(h[0] + dt * (q3 - q1 + g1 * k1 * v1 - leak) / A[0], 0.0),
            max(h[1] + dt * (q4 - q2 + g2 * k2 * v2) / A[1], 0.0),
            max(h[2] + dt * (-q3 + (1 - g2) * k2 * v2) / A[2], 0.0),
            max(h[3] + dt * (-q4 + (1 - g1) * k1 * v1) / A[3], 0.0),
        ]
        rows.append({
            "level_1": h[0] + rng.normal(0.02), "level_2": h[1] + rng.normal(0.02),
            "level_3": h[2] + rng.normal(0.02), "level_4": h[3] + rng.normal(0.02),
            "v1": v1 + rng.normal(0.01), "v2": v2 + rng.normal(0.01),
            "label": 1 if k >= onset else 0, "phase": 0 if k < 120 else 1,
        })
    roles = {
        "dataset": "quadruple_tank_instrumented", "kind": "simulation",
        "description": "Johansson quadruple-tank; mass-balance witness; unmeasured leak in tank 1.",
        "variables": [
            {"name": "level_1", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "level_2", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "level_3", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "level_4", "role": "measured", "unit": "cm", "quantity": "level"},
            {"name": "v1", "role": "manipulated", "unit": "V", "quantity": "pump"},
            {"name": "v2", "role": "manipulated", "unit": "V", "quantity": "pump"},
        ],
        "balance": {
            "type": "mass_quad_tank", "A1": A[0], "a1": a[0], "a3": a[2], "g": g, "k1": k1, "gamma1": g1,
            "dt_s": dt, "level_1": "level_1", "level_3": "level_3", "v1": "v1",
            "definition": "residual = A1*dh1/dt - (a3*sqrt(2g*h3) - a1*sqrt(2g*h1) + gamma1*k1*v1)",
        },
        "control_action": {"manipulated": "v1", "controlled": "level_1", "saturation_high": 12.0},
        "fault": {"onset_index": onset, "mode": "unmeasured leak in tank 1"},
    }
    return rows, roles


def quad_tank_closure(rows, roles):
    b = roles["balance"]
    A1, a1, a3, g, k1, g1, dt = b["A1"], b["a1"], b["a3"], b["g"], b["k1"], b["gamma1"], b["dt_s"]
    onset = roles["fault"]["onset_index"]
    sq = lambda x: math.sqrt(max(x, 0.0))
    nom, flt = [], []
    for k in range(1, len(rows)):
        h1, h3, v1 = rows[k]["level_1"], rows[k]["level_3"], rows[k]["v1"]
        dh1 = (rows[k]["level_1"] - rows[k - 1]["level_1"]) / dt
        res = A1 * dh1 - (a3 * sq(2 * g * h3) - a1 * sq(2 * g * h1) + g1 * k1 * v1)
        (flt if k >= onset else nom).append(abs(res))
    return sum(nom) / len(nom), sum(flt) / len(flt)


# ──────────────────────────────────────────────────────────────────────────────
# CSTH (continuous stirred tank heater): energy-balance witness (insulation failure)
# ──────────────────────────────────────────────────────────────────────────────
def gen_csth(n=720, onset=420, seed=41):
    """Continuous Stirred Tank Heater (Thornhill-style). Stirred tank with cold-water inflow Fc,
    steam duty Q_steam (manipulated; PI temperature control), and a level-holding outflow. ENERGY
    balance rho_cp*V dT/dt = rho_cp*Fc*(Tc - T) + Q_steam - h_amb*(T - Tamb) - Q_loss, with a known
    ambient-loss term h_amb*(T-Tamb). At `onset` an INSULATION failure opens an unmodelled extra loss
    Q_loss (grows), breaking the energy-balance closure -- the energy analogue of an unmeasured leak.

    Columns: inflow_cold [L/min], temp_cold_in [degC], tank_temp [degC], steam_duty [kJ/min,
    MANIPULATED]; label, phase. All energy terms in kJ/min; rho_cp = 4.18 kJ/(L*K).
    """
    V = 8.0
    rho_cp = 4.18
    h_amb, Tamb = 5.0, 20.0
    Fc, Tc, sp = 6.0, 20.0, 70.0
    kp, ki, dt = 60.0, 2.0, 0.2
    rng = Rng(seed)
    T, integ = 60.0, 0.0
    rows = []
    for k in range(n):
        e = sp - T
        integ += e * dt
        qs = max(0.0, min(4000.0, 1500.0 + kp * e + ki * integ))   # steam duty [kJ/min]
        q_loss = 12.0 * (k - onset) if k >= onset else 0.0          # unmodelled extra loss [kJ/min]
        dT = (rho_cp * Fc * (Tc - T) + qs - h_amb * (T - Tamb) - q_loss) / (rho_cp * V)
        T += dt * dT
        rows.append({
            "inflow_cold": Fc + rng.normal(0.02),
            "temp_cold_in": Tc + rng.normal(0.05),
            "tank_temp": T + rng.normal(0.03),
            "steam_duty": qs + rng.normal(2.0),
            "label": 1 if k >= onset else 0, "phase": 0 if k < 120 else 1,
        })
    roles = {
        "dataset": "csth_instrumented", "kind": "simulation",
        "description": "Continuous stirred tank heater; energy-balance witness; insulation-failure heat loss.",
        "variables": [
            {"name": "inflow_cold", "role": "measured", "unit": "L/min", "quantity": "flow"},
            {"name": "temp_cold_in", "role": "measured", "unit": "degC", "quantity": "temperature"},
            {"name": "tank_temp", "role": "controlled", "unit": "degC", "quantity": "temperature"},
            {"name": "steam_duty", "role": "manipulated", "unit": "kJ/min", "quantity": "heat-duty"},
        ],
        "balance": {
            "type": "energy_csth", "V_L": V, "rho_cp_kJ_per_LK": rho_cp, "h_amb_kJ_per_minK": h_amb,
            "Tamb_degC": Tamb, "dt_min": dt,
            "tank_temp": "tank_temp", "temp_cold_in": "temp_cold_in", "inflow_cold": "inflow_cold",
            "steam_duty": "steam_duty",
            "definition": "residual = rho_cp*V*dT/dt - (rho_cp*Fc*(Tc-T) + Q_steam - h_amb*(T-Tamb))",
        },
        "control_action": {"manipulated": "steam_duty", "controlled": "tank_temp", "saturation_high": 4000.0},
        "fault": {"onset_index": onset, "mode": "insulation failure (unmodelled heat loss)"},
    }
    return rows, roles


def csth_closure(rows, roles):
    b = roles["balance"]
    V, rho_cp, h_amb, Tamb, dt = b["V_L"], b["rho_cp_kJ_per_LK"], b["h_amb_kJ_per_minK"], b["Tamb_degC"], b["dt_min"]
    onset = roles["fault"]["onset_index"]
    nom, flt = [], []
    for k in range(1, len(rows)):
        T = rows[k]["tank_temp"]
        dT = (rows[k]["tank_temp"] - rows[k - 1]["tank_temp"]) / dt
        res = rho_cp * V * dT - (rho_cp * rows[k]["inflow_cold"] * (rows[k]["temp_cold_in"] - T) + rows[k]["steam_duty"] - h_amb * (T - Tamb))
        (flt if k >= onset else nom).append(abs(res))
    return sum(nom) / len(nom), sum(flt) / len(flt)


# ──────────────────────────────────────────────────────────────────────────────
# Valve stiction (F1) and valve hunting (F9): control-loop fault DEMONSTRATORS.
# These are NOT balance witnesses — they exhibit the F1/F9 residual motifs on the cheap PV/OP
# sensors and are detected by the standard atlas detector bank (spectral-entropy + Page-Hinkley on
# the SPE residual). They carry no roles sidecar (no balance to recompute); the "executed" evidence
# is the committed CSV + the gated test `tests/fault_demonstrators.rs`, which runs them through
# `pipeline::analyze` and asserts the signature fires at the labelled onset.
# ──────────────────────────────────────────────────────────────────────────────
# A benign, bounded operating-point dither for the controller output OP. It is the sum of two FAST
# components (periods 9 and 13 samples, both shorter than the detectors' drift/spectral windows), so it
# reads as stationary structure — NOT a trend — and gives PV/OP genuine, well-conditioned range. The
# valve gain is near-static (small tau), so a HEALTHY stem makes PV almost collinear with OP: the
# baseline SPE residual is low and well-defined (not degenerate, not trend-flagged). The fault is then a
# clean structural break of that collinearity.
def _op_dither(k):
    return 50.0 + 7.0 * math.sin(2.0 * math.pi * k / 9.0) + 4.0 * math.sin(2.0 * math.pi * k / 13.0)


def gen_valve_stiction(n=720, onset=360, seed=53):
    """Near-static valve `tau dPV/dt = -PV + K*stem` (small tau) driven by a benign OP dither. Healthy:
    the stem follows OP, so PV is near-collinear with OP and the baseline SPE residual is low. After
    `onset` the valve STICKS — the stem moves only when |OP - stem| exceeds the stiction band S, then
    slips to OP — so PV stalls while OP moves (stick) then jumps (slip): the textbook stick-slip limit
    cycle, breaking PV/OP collinearity into the F1 SPE-residual signature. Columns: pv, op, label, phase."""
    tau, K, dt = 1.5, 1.0, 1.0
    S = 5.0  # stiction band (deadband + stickband), engaged only after onset
    rng = Rng(seed)
    pv, stem = 50.0, 50.0
    rows, slips = [], 0
    for k in range(n):
        op = _op_dither(k)
        if k >= onset:
            if abs(op - stem) > S:  # overcome static friction -> slip to the commanded position
                stem = op
                slips += 1
            # else: stick (stem unchanged) while OP keeps moving
        else:
            stem = op  # healthy valve: stem tracks OP
        pv += dt * (-pv + K * stem) / tau + rng.normal(0.05)
        rows.append({"pv": pv, "op": op, "label": 1 if k >= onset else 0, "phase": 0 if k < onset else 1})
    return rows, slips


def gen_valve_hunting(n=720, onset=360, seed=67):
    """Same near-static valve driven by the same benign OP dither, but after `onset` the positioner
    HUNTS: a fixed-period oscillation rides on top of OP (`stem = OP + hunt`), so PV gains a sustained
    limit-cycle component OP does not have — a dominant spectral peak in the PV/OP SPE residual (the F9
    signature). Distinct from stiction's irregular stick-slip: a clean fixed period. Columns: pv, op,
    label, phase."""
    tau, K, dt = 1.5, 1.0, 1.0
    period, amp = 20.0, 10.0
    rng = Rng(seed)
    pv = 50.0
    rows = []
    for k in range(n):
        op = _op_dither(k)
        hunt = amp * math.sin(2.0 * math.pi * k / period) if k >= onset else 0.0
        stem = op + hunt  # positioner hunts on top of OP
        pv += dt * (-pv + K * stem) / tau + rng.normal(0.05)
        rows.append({"pv": pv, "op": op, "label": 1 if k >= onset else 0, "phase": 0 if k < onset else 1})
    return rows, period


def gen_heat_fouling(n=720, onset=360, seed=71):
    """F3 (heat-transfer fouling). A cooler outlet temperature tracks a benign stationary heat-duty
    dither (well-conditioned baseline, PV/OP-collinear). After `onset` the heat-transfer effectiveness
    degrades **monotonically** (fouling), so the outlet temperature drifts steadily away from the
    duty-line — a slow `DriftAccum` with a positive Mann-Kendall trend and **no sharp slew** (the F3
    signature), which lands in the PV/OP SPE residual. Columns: heat_duty, temp_out, label, phase."""
    tau, dt = 1.5, 1.0
    rng = Rng(seed)
    temp_out = 50.0
    rows = []
    for k in range(n):
        duty = _op_dither(k)  # benign stationary heat-duty proxy (conditions the PCA, no trend)
        # Effectiveness loss ramps linearly after onset: the outlet temp can no longer follow the duty
        # and drifts monotonically above the calibrated duty->temp relationship.
        foul = 0.05 * float(k - onset) if k >= onset else 0.0
        target = duty + foul
        temp_out += dt * (-temp_out + target) / tau + rng.normal(0.05)
        rows.append({"heat_duty": duty, "temp_out": temp_out, "label": 1 if k >= onset else 0, "phase": 0 if k < onset else 1})
    return rows


def gen_controller_masking(n=720, onset=360, seed=83):
    """F8 (controller-compensation masking). A PI loop holds the controlled variable CV at setpoint
    against a benign stationary load (well-conditioned baseline). After `onset` a load disturbance grows
    **monotonically**; the controller ramps the manipulated variable MV to keep CV in-band, so CV stays
    flat while MV drifts — the masking motif (latent MV/CV structural change before any raw CV-limit
    breach). The growing MV-vs-flat-CV joint shift lands in the T^2/SPE residual. Columns: cv, mv, label,
    phase."""
    tau, k_gain, sp, dt = 1.5, 1.0, 50.0, 1.0
    kp, ki = 0.9, 0.10
    rng = Rng(seed)
    cv, integ, load = 50.0, 0.0, 0.0
    rows = []
    for k in range(n):
        # Benign stationary load dither (both regions) the loop rejects, plus a slow MONOTONE load growth
        # after onset that the controller masks at the CV by ramping MV.
        load = 0.7 * load + rng.normal(2.0) + (0.05 * float(k - onset) if k >= onset else 0.0)
        err = sp - cv
        integ += err * dt
        mv = max(0.0, min(100.0, 50.0 + kp * err + ki * integ))
        cv += dt * (-cv + k_gain * mv - load) / tau + rng.normal(0.05)
        rows.append({"cv": cv, "mv": mv, "label": 1 if k >= onset else 0, "phase": 0 if k < onset else 1})
    return rows


def gen_cavitation(n=720, onset=360, seed=97):
    """F2 (pump cavitation). A centrifugal pump runs against a benign stationary suction/flow dither.
    The dither dominates the variance, so PCA keeps a single component (the flow direction) and the
    discharge pressure / flow / motor current ride that pump-curve mode collinearly — leaving the
    vibration channel as the SPE residual.

    In the healthy region the vibration RMS is a low, slow tonal "hum" (period ~45 samples) plus a
    tiny noise floor: a structured, LOW-spectral-entropy baseline SPE. After `onset` the suction
    pressure drops into cavitation — vapour pockets form and collapse erratically — so the vibration
    RMS rises and gains strong BROADBAND high-frequency energy (modelled as white noise, whose flat
    spectrum is high spectral entropy), while the discharge pressure and flow sag and fluctuate and the
    motor current hunts. The broadband energy breaks the learned pump-curve correlation, producing a
    large, noisy SPE residual whose spectral entropy jumps from tonal (baseline) to broadband (fault).

    That contrast is exactly what the SPE-derived detectors key on: ewma_spe (sustained magnitude
    rise), knn_spe (samples far from the baseline SPE cloud) and spectral_entropy_spe (tonal→broadband
    shape change) — the F2 signature. The broadband-vs-narrowband distinction is what discriminates
    cavitation from a discrete bearing-defect tone (F5) or a narrowband imbalance (F11).

    Columns: vibration_rms, discharge_pressure, flow, motor_current, label, phase.

    PCA note: the pipeline standardises every channel to unit variance before PCA, so a vibration
    channel left *independent* of the flow dither would form its own retained principal component —
    capturing the cavitation energy into T² (scores) instead of the SPE residual the F2 detectors read.
    To keep the energy in SPE we make the healthy vibration RMS *track the dither* (gain `kqv`) so all
    four channels share one dominant correlation direction (PCA keeps k=1), and add only a small slow
    `hum` orthogonal to that direction. The hum is the dominant component of the baseline SPE residual
    (a low-frequency tone, low spectral entropy); cavitation's broadband white energy then arrives as a
    new, large, high-entropy SPE direction the baseline PCA never modelled."""
    v0, p0, q0, i0 = 5.0, 60.0, 50.0, 40.0  # healthy operating point: vib RMS, discharge P, flow, motor I
    kqp, kqi = 0.6, 0.5  # pump-curve gains: flow dither couples into discharge pressure and motor current
    kqv = 0.30  # vibration-to-dither gain: keeps vibration on the dominant PCA mode (so k=1, energy → SPE)
    rng = Rng(seed)
    rows = []
    for k in range(n):
        d = _op_dither(k) - 50.0  # zero-mean benign suction/flow dither — drives the dominant PCA mode
        # Narrowband tonal pump hum on a short period (6 samples) at small amplitude, orthogonal to the
        # flow dither (periods 9/13) so it survives PCA into the SPE residual: a structured,
        # LOW-spectral-entropy baseline (the contrast cavitation later breaks by going broadband). The
        # short period means every 32-sample window contains several full cycles, so the SPE *value*
        # distribution and trend are stationary from the start — this keeps the distributional/trend
        # detectors (psi/ks/mann_kendall) quiet through the baseline (no spurious pre-onset episode),
        # while the concentrated spectral peak keeps the window entropy low.
        hum = 0.20 * math.sin(2.0 * math.pi * k / 6.0)
        if k < onset:
            vib = v0 + kqv * d + hum + rng.normal(0.02)
            press = p0 - kqp * d + rng.normal(0.02)
            flow = q0 + d + rng.normal(0.02)
            curr = i0 + kqi * d + rng.normal(0.02)
        else:
            # Cavitation: broadband (white) vibration energy + a DC RMS rise; pressure/flow sag with
            # erratic fluctuation; motor current hunts. White noise → flat spectrum → high entropy, and
            # it is orthogonal to the dither mode so it lands squarely in the SPE residual.
            vib = v0 + kqv * d + hum + 2.0 + rng.normal(2.5)
            press = p0 - kqp * d - 4.0 + rng.normal(2.0)
            flow = q0 + d - 3.0 + rng.normal(1.5)
            curr = i0 + kqi * d + rng.normal(1.5)
        rows.append({"vibration_rms": vib, "discharge_pressure": press, "flow": flow,
                     "motor_current": curr, "label": 1 if k >= onset else 0,
                     "phase": 0 if k < onset else 1})
    return rows


def osc_amplitude(rows, onset, key="pv"):
    """Validation helper: peak-to-peak amplitude of `key` in the nominal vs fault regions."""
    nom = [r[key] for r in rows[:onset]]
    flt = [r[key] for r in rows[onset:]]
    ptp = lambda xs: (max(xs) - min(xs)) if xs else 0.0
    return ptp(nom), ptp(flt)


def write_fault_slice(name, rows, cols):
    """Write a detector-bank demonstrator CSV (no roles sidecar — these are not balance witnesses)."""
    csv_path = os.path.join(OUT, f"{name}.csv")
    with open(csv_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(cols)
        for r in rows:
            w.writerow([f"{r[c]:.6f}" if isinstance(r[c], float) else r[c] for c in cols])
    return csv_path


def write_slice(name, rows, roles):
    cols = [v["name"] for v in roles["variables"]] + ["label", "phase"]
    csv_path = os.path.join(OUT, f"{name}.csv")
    with open(csv_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(cols)
        for r in rows:
            w.writerow([f"{r[c]:.6f}" if isinstance(r[c], float) else r[c] for c in cols])
    with open(os.path.join(OUT, f"{name}.roles.json"), "w") as f:
        json.dump(roles, f, indent=2)
    return csv_path


def main():
    tt_rows, tt_roles = gen_three_tank()
    write_slice("three_tank_instrumented", tt_rows, tt_roles)
    n_nom, n_flt = three_tank_closure(tt_rows, tt_roles)
    print(f"three_tank_instrumented: mass-balance closure |res| nominal={n_nom:.3f} fault={n_flt:.3f} "
          f"(ratio {n_flt / max(n_nom, 1e-9):.1f}x)")

    cstr_rows, cstr_roles = gen_cstr()
    write_slice("cstr_instrumented", cstr_rows, cstr_roles)
    c_nom, c_flt = cstr_closure(cstr_rows, cstr_roles)
    print(f"cstr_instrumented:       energy-balance closure |res| nominal={c_nom:.1f} fault={c_flt:.1f} "
          f"(ratio {c_flt / max(c_nom, 1e-9):.1f}x)")

    qt_rows, qt_roles = gen_quadruple_tank()
    write_slice("quadruple_tank_instrumented", qt_rows, qt_roles)
    q_nom, q_flt = quad_tank_closure(qt_rows, qt_roles)
    print(f"quadruple_tank_instr.:   mass-balance closure |res| nominal={q_nom:.3f} fault={q_flt:.3f} "
          f"(ratio {q_flt / max(q_nom, 1e-9):.1f}x)")

    ch_rows, ch_roles = gen_csth()
    write_slice("csth_instrumented", ch_rows, ch_roles)
    h_nom, h_flt = csth_closure(ch_rows, ch_roles)
    print(f"csth_instrumented:       energy-balance closure |res| nominal={h_nom:.2f} fault={h_flt:.2f} "
          f"(ratio {h_flt / max(h_nom, 1e-9):.1f}x)")

    # F1 / F9 control-loop demonstrators (detector-bank, no roles sidecar).
    st_rows, slips = gen_valve_stiction()
    write_fault_slice("valve_stiction_instrumented", st_rows, ["pv", "op", "label", "phase"])
    s_nom, s_flt = osc_amplitude(st_rows, 360, "pv")
    print(f"valve_stiction_instr.:   PV peak-to-peak nominal={s_nom:.2f} fault={s_flt:.2f} "
          f"({slips} slip events) -- F1 stick-slip limit cycle")

    hu_rows, period = gen_valve_hunting()
    write_fault_slice("valve_hunting_instrumented", hu_rows, ["pv", "op", "label", "phase"])
    u_nom, u_flt = osc_amplitude(hu_rows, 360, "pv")
    print(f"valve_hunting_instr.:    PV peak-to-peak nominal={u_nom:.2f} fault={u_flt:.2f} "
          f"(period {period:.0f} samples) -- F9 limit-cycle oscillation")

    hf_rows = gen_heat_fouling()
    write_fault_slice("heat_fouling_instrumented", hf_rows, ["heat_duty", "temp_out", "label", "phase"])
    hf_nom, hf_flt = osc_amplitude(hf_rows, 360, "temp_out")
    print(f"heat_fouling_instr.:     temp_out peak-to-peak nominal={hf_nom:.2f} fault={hf_flt:.2f} "
          f"-- F3 monotone fouling drift")

    cm_rows = gen_controller_masking()
    write_fault_slice("controller_masking_instrumented", cm_rows, ["cv", "mv", "label", "phase"])
    cm_cv = osc_amplitude(cm_rows, 360, "cv")
    cm_mv = osc_amplitude(cm_rows, 360, "mv")
    print(f"controller_masking_instr.: CV ptp nom={cm_cv[0]:.2f}/flt={cm_cv[1]:.2f} (held), "
          f"MV ptp nom={cm_mv[0]:.2f}/flt={cm_mv[1]:.2f} (drifts) -- F8 masking motif")

    cv_rows = gen_cavitation()
    write_fault_slice("cavitation_instrumented", cv_rows,
                      ["vibration_rms", "discharge_pressure", "flow", "motor_current", "label", "phase"])
    cv_vib = osc_amplitude(cv_rows, 360, "vibration_rms")
    print(f"cavitation_instr.:       vibration_rms peak-to-peak nominal={cv_vib[0]:.2f} fault={cv_vib[1]:.2f} "
          f"-- F2 broadband cavitation energy")
    print(f"wrote slices + roles sidecars to {OUT}")


if __name__ == "__main__":
    main()
