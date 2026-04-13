use std::string::{String, ToString};
use std::vec::Vec;
use serde::Deserialize;

use crate::{
    drilling::DrillingFrame,
    drilling_real::VolveFrame,
    error::DsfbError,
    oilwell::OilwellFrame,
    pipeline::PipelineFrame,
    rotating::RotatingFrame,
    rotating_real::EspFrame,
    subsea::SubseaFrame,
};

fn parse_f64(field: &'static str, value: &str) -> Result<f64, DsfbError> {
    value.trim().parse::<f64>()
        .map_err(|_| DsfbError::ParseFloat { field, value: value.to_string() })
}

#[derive(Debug, Deserialize)]
pub struct PipelineRow {
    pub timestamp: String,
    pub expected_flow_balance: String,
    pub observed_flow_balance: String,
    pub inlet_pressure: String,
    pub outlet_pressure: String,
}

#[derive(Debug, Deserialize)]
pub struct RotatingRow {
    pub timestamp: String,
    pub expected_head: String,
    pub observed_head: String,
    pub vibration_rms: String,
    pub flow_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct DrillingRow {
    pub timestamp: String,
    pub expected_torque: String,
    pub observed_torque: String,
    pub wob: String,
    pub rpm: String,
}

#[derive(Debug, Deserialize)]
pub struct SubseaRow {
    pub timestamp: String,
    pub expected_actuation_pressure: String,
    pub observed_actuation_pressure: String,
    pub valve_command: String,
}

pub fn load_pipeline_csv(path: &str) -> Result<Vec<PipelineFrame>, DsfbError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<PipelineRow>() {
        let row = row?;
        out.push(PipelineFrame {
            timestamp: parse_f64("timestamp", &row.timestamp)?,
            expected_flow_balance: parse_f64("expected_flow_balance", &row.expected_flow_balance)?,
            observed_flow_balance: parse_f64("observed_flow_balance", &row.observed_flow_balance)?,
            inlet_pressure: parse_f64("inlet_pressure", &row.inlet_pressure)?,
            outlet_pressure: parse_f64("outlet_pressure", &row.outlet_pressure)?,
        });
    }
    Ok(out)
}

pub fn load_rotating_csv(path: &str) -> Result<Vec<RotatingFrame>, DsfbError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<RotatingRow>() {
        let row = row?;
        out.push(RotatingFrame {
            timestamp: parse_f64("timestamp", &row.timestamp)?,
            expected_head: parse_f64("expected_head", &row.expected_head)?,
            observed_head: parse_f64("observed_head", &row.observed_head)?,
            vibration_rms: parse_f64("vibration_rms", &row.vibration_rms)?,
            flow_rate: parse_f64("flow_rate", &row.flow_rate)?,
        });
    }
    Ok(out)
}

pub fn load_drilling_csv(path: &str) -> Result<Vec<DrillingFrame>, DsfbError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<DrillingRow>() {
        let row = row?;
        out.push(DrillingFrame {
            timestamp: parse_f64("timestamp", &row.timestamp)?,
            expected_torque: parse_f64("expected_torque", &row.expected_torque)?,
            observed_torque: parse_f64("observed_torque", &row.observed_torque)?,
            wob: parse_f64("wob", &row.wob)?,
            rpm: parse_f64("rpm", &row.rpm)?,
        });
    }
    Ok(out)
}

pub fn load_subsea_csv(path: &str) -> Result<Vec<SubseaFrame>, DsfbError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<SubseaRow>() {
        let row = row?;
        out.push(SubseaFrame {
            timestamp: parse_f64("timestamp", &row.timestamp)?,
            expected_actuation_pressure: parse_f64("expected_actuation_pressure", &row.expected_actuation_pressure)?,
            observed_actuation_pressure: parse_f64("observed_actuation_pressure", &row.observed_actuation_pressure)?,
            valve_command: parse_f64("valve_command", &row.valve_command)?,
        });
    }
    Ok(out)
}

// ── Real Petrobras 3W oilwell production data ─────────────────────────────────
// Source: github.com/petrobras/3W v2.0.0, CC BY 4.0.
// Only real WELL-* instances used; SIMULATED_* and DRAWN_* excluded.

#[derive(Debug, Deserialize)]
pub struct OilwellRow {
    pub timestamp: String,
    pub episode_id: String,
    pub episode: String,
    pub event_type: String,
    pub event_class: String,
    pub expected_annulus_pa: String,
    pub observed_annulus_pa: String,
    pub expected_choke_pa: String,
    pub observed_choke_pa: String,
    pub expected_xmas_pa: String,
    pub observed_xmas_pa: String,
    pub expected_xmas_degc: String,
    pub observed_xmas_degc: String,
}

/// Parse a field that may be "NaN" or a valid f64; returns 0.0 on NaN.
fn parse_f64_nullable(field: &'static str, value: &str) -> Result<f64, DsfbError> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("nan") || v.is_empty() {
        Ok(0.0)
    } else {
        parse_f64(field, v)
    }
}

pub fn load_oilwell_csv(path: &str) -> Result<Vec<OilwellFrame>, DsfbError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<OilwellRow>() {
        let row = row?;
        // Skip rows with no primary choke-pressure observation
        let obs_choke = parse_f64_nullable("observed_choke_pa", &row.observed_choke_pa)?;
        let exp_choke = parse_f64_nullable("expected_choke_pa", &row.expected_choke_pa)?;
        if obs_choke == 0.0 && exp_choke == 0.0 {
            continue;
        }
        let event_class = row.event_class.trim().parse::<i16>()
            .unwrap_or(0);
        out.push(OilwellFrame {
            timestamp: parse_f64("timestamp", &row.timestamp)?,
            expected_choke_pa: exp_choke,
            observed_choke_pa: obs_choke,
            observed_annulus_pa: parse_f64_nullable("observed_annulus_pa", &row.observed_annulus_pa)?,
            observed_xmas_pa: parse_f64_nullable("observed_xmas_pa", &row.observed_xmas_pa)?,
            observed_xmas_degc: parse_f64_nullable("observed_xmas_degc", &row.observed_xmas_degc)?,
            event_class,
        });
    }
    Ok(out)
}

// ── Real Equinor Volve depth-indexed drilling data ────────────────────────────
// Source: Equinor Volve Data Village, well 15/9-F-15 (WITSML 1.4.1 logs).
// Equinor Volve Data Licence V1.0. Channel TQA = surface torque [kNm].
// SWOB/HKLD converted from source kkgf (×9.80665). Resampled to 0.5-m steps.

#[derive(Debug, Deserialize)]
pub struct VolveRow {
    pub depth_m: String,
    pub well: String,
    #[serde(rename = "TQA")]
    pub tqa: String,
    #[serde(rename = "SWOB")]
    pub swob: String,
    #[serde(rename = "RPM")]
    pub rpm: String,
    #[serde(rename = "HKLD")]
    pub hkld: String,
    #[serde(rename = "SPPA")]
    pub sppa: String,
    #[serde(rename = "baseline_TQA")]
    pub baseline_tqa: String,
    #[serde(rename = "baseline_SWOB")]
    pub baseline_swob: String,
    #[serde(rename = "baseline_RPM")]
    pub baseline_rpm: String,
    #[serde(rename = "baseline_HKLD")]
    pub baseline_hkld: String,
    #[serde(rename = "baseline_SPPA")]
    pub baseline_sppa: String,
}

pub fn load_volve_csv(path: &str) -> Result<Vec<VolveFrame>, DsfbError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<VolveRow>() {
        let row = row?;
        let obs_tqa = parse_f64_nullable("TQA", &row.tqa)?;
        let base_tqa = parse_f64_nullable("baseline_TQA", &row.baseline_tqa)?;
        out.push(VolveFrame {
            depth_m: parse_f64("depth_m", &row.depth_m)?,
            baseline_tqa_knm: base_tqa,
            observed_tqa_knm: obs_tqa,
            swob_kn: parse_f64_nullable("SWOB", &row.swob)?,
            rpm: parse_f64_nullable("RPM", &row.rpm)?,
            hkld_kn: parse_f64_nullable("HKLD", &row.hkld)?,
            sppa_kpa: parse_f64_nullable("SPPA", &row.sppa)?,
        });
    }
    Ok(out)
}

// ── Real RPDBCS ESP vibration dataset ────────────────────────────────────────
// Source: RPDBCS ESPset — 11 ESP units, 6 032 vibration snapshots,
//         5 fault classes.  MIT License.

#[derive(Debug, Deserialize)]
pub struct EspRow {
    pub step: String,
    pub esp_id: String,
    pub label: String,
    pub rms_broadband: String,
    pub peak1x: String,
    pub peak2x: String,
    pub median_8_13hz: String,
    pub coeff_a: String,
    pub coeff_b: String,
    pub baseline_rms: String,
    pub baseline_peak1x: String,
}

pub fn load_esp_csv(path: &str) -> Result<Vec<EspFrame>, DsfbError> {
    use crate::rotating_real::EspFrame;
    let mut rdr = csv::Reader::from_path(path)?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<EspRow>() {
        let row = row?;
        let step = row.step.trim().parse::<u32>().unwrap_or(0);
        let esp_id = row.esp_id.trim().parse::<u8>().unwrap_or(0);
        out.push(EspFrame {
            step,
            esp_id,
            baseline_rms:    parse_f64_nullable("baseline_rms",    &row.baseline_rms)?,
            rms_broadband:   parse_f64_nullable("rms_broadband",   &row.rms_broadband)?,
            peak1x:          parse_f64_nullable("peak1x",          &row.peak1x)?,
            peak2x:          parse_f64_nullable("peak2x",          &row.peak2x)?,
            baseline_peak1x: parse_f64_nullable("baseline_peak1x", &row.baseline_peak1x)?,
            median_8_13hz:   parse_f64_nullable("median_8_13hz",   &row.median_8_13hz)?,
            coeff_a:         parse_f64_nullable("coeff_a",         &row.coeff_a)?,
            coeff_b:         parse_f64_nullable("coeff_b",         &row.coeff_b)?,
        });
    }
    Ok(out)
}
