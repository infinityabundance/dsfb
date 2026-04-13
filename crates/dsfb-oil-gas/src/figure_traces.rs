use crate::{
    load_volve_csv, AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier,
    GrammarState, ResidualSample,
};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{boxed::Box, string::String, vec::Vec};

pub struct GrammarTraceExport {
    pub trace_dir: PathBuf,
    pub steps_3w: usize,
    pub steps_volve: usize,
    pub steps_esp: usize,
}

pub fn export_grammar_traces(crate_root: &Path) -> Result<GrammarTraceExport, Box<dyn std::error::Error>> {
    let trace_dir = crate_root.join("figures").join("trace_data");
    fs::create_dir_all(&trace_dir)?;

    let steps_3w = export_3w(crate_root, &trace_dir)?;
    let steps_volve = export_volve(crate_root, &trace_dir)?;
    let steps_esp = export_esp(crate_root, &trace_dir)?;

    Ok(GrammarTraceExport {
        trace_dir,
        steps_3w,
        steps_volve,
        steps_esp,
    })
}

fn tok(state: GrammarState) -> &'static str {
    match state {
        GrammarState::Nominal => "Nominal",
        GrammarState::DriftAccum => "DriftAccum",
        GrammarState::SlewSpike => "SlewSpike",
        GrammarState::EnvViolation => "EnvViolation",
        GrammarState::BoundaryGrazing => "BoundaryGrazing",
        GrammarState::Recovery => "Recovery",
        GrammarState::Compound => "Compound",
        GrammarState::SensorFault => "SensorFault",
    }
}

fn f64_or(input: &str, fallback: f64) -> f64 {
    let input = input.trim();
    if input.eq_ignore_ascii_case("nan") || input.is_empty() {
        fallback
    } else {
        input.parse().unwrap_or(fallback)
    }
}

#[derive(Debug, Deserialize)]
struct RawOilwellRow {
    timestamp: String,
    episode_id: String,
    episode: String,
    #[allow(dead_code)]
    event_type: String,
    event_class: String,
    #[allow(dead_code)]
    expected_annulus_pa: String,
    #[allow(dead_code)]
    observed_annulus_pa: String,
    expected_choke_pa: String,
    observed_choke_pa: String,
    #[allow(dead_code)]
    expected_xmas_pa: String,
    #[allow(dead_code)]
    observed_xmas_pa: String,
    #[allow(dead_code)]
    expected_xmas_degc: String,
    #[allow(dead_code)]
    observed_xmas_degc: String,
}

#[derive(Debug, Deserialize)]
struct RawEspRow {
    step: String,
    esp_id: String,
    label: String,
    rms_broadband: String,
    peak1x: String,
    peak2x: String,
    #[allow(dead_code)]
    median_8_13hz: String,
    #[allow(dead_code)]
    coeff_a: String,
    #[allow(dead_code)]
    coeff_b: String,
    baseline_rms: String,
    #[allow(dead_code)]
    baseline_peak1x: String,
}

fn export_3w(crate_root: &Path, trace_dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let env = AdmissibilityEnvelope::default_oilwell();
    let r_max = env.r_max.abs().max(env.r_min.abs());
    let d_max = env.delta_max.abs().max(env.delta_min.abs());
    let s_max = env.sigma_max.abs().max(env.sigma_min.abs());

    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    #[derive(Debug)]
    struct ValidRow {
        ts: f64,
        ep_id: String,
        ep_name: String,
        ec: i16,
        obs: f64,
        exp: f64,
    }

    let mut rdr = csv::Reader::from_path(crate_root.join("data").join("oilwell_real.csv"))?;
    let mut valid: Vec<ValidRow> = Vec::new();

    for result in rdr.deserialize::<RawOilwellRow>() {
        let row = result?;
        let obs = f64_or(&row.observed_choke_pa, 0.0);
        let exp = f64_or(&row.expected_choke_pa, 0.0);
        if obs == 0.0 && exp == 0.0 {
            continue;
        }

        let ts = f64_or(&row.timestamp, valid.len() as f64 * 60.0);
        let ec = row.event_class.trim().parse::<i16>().unwrap_or(0);

        let sample = ResidualSample::new(ts, obs - exp, 0.0, "choke_pa");
        engine.ingest_sample(&sample);

        valid.push(ValidRow {
            ts,
            ep_id: row.episode_id,
            ep_name: row.episode,
            ec,
            obs,
            exp,
        });
    }

    let history = engine.history();
    let mut out = File::create(trace_dir.join("real_3w_trace.csv"))?;
    writeln!(
        out,
        "step_idx,timestamp_s,episode_id,episode_name,event_class,\
         observed_pa,expected_pa,residual_pa,drift_pa,slew_pa,token,\
         r_norm,delta_norm,sigma_norm"
    )?;

    assert_eq!(valid.len(), history.len(), "3W: row/history count mismatch");
    let count = valid.len();

    for (idx, (row, hist)) in valid.iter().zip(history.iter()).enumerate() {
        let r = hist.triple.r;
        let d = hist.triple.delta;
        let s = hist.triple.sigma;
        let rn = if r_max.abs() > 1e-12 { r / r_max } else { 0.0 };
        let dn = if d_max.abs() > 1e-12 { d / d_max } else { 0.0 };
        let sn = if s_max.abs() > 1e-12 { s / s_max } else { 0.0 };

        writeln!(
            out,
            "{idx},{:.0},{},{},{},{:.0},{:.0},{:.2},{:.4},{:.4},{},{:.6},{:.6},{:.6}",
            row.ts,
            row.ep_id,
            row.ep_name,
            row.ec,
            row.obs,
            row.exp,
            r,
            d,
            s,
            tok(hist.state),
            rn,
            dn,
            sn,
        )?;
    }

    write_env(trace_dir.join("env_3w.csv"), &env)?;
    Ok(count)
}

fn export_volve(crate_root: &Path, trace_dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let env = AdmissibilityEnvelope::default_volve_drilling();
    let r_max = env.r_max.abs().max(env.r_min.abs());
    let d_max = env.delta_max.abs().max(env.delta_min.abs());
    let s_max = env.sigma_max.abs().max(env.sigma_min.abs());

    let data_path = crate_root.join("data").join("drilling_real.csv");
    let frames = load_volve_csv(data_path.to_string_lossy().as_ref())?;
    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());

    for frame in &frames {
        let residual = frame.observed_tqa_knm - frame.baseline_tqa_knm;
        let sample = ResidualSample::new(frame.depth_m, residual, 0.0, "tqa_knm");
        engine.ingest_sample(&sample);
    }

    let history = engine.history();
    let mut out = File::create(trace_dir.join("real_volve_trace.csv"))?;
    writeln!(
        out,
        "step_idx,depth_m,observed_tqa_knm,baseline_tqa_knm,swob_kn,rpm,hkld_kn,sppa_kpa,\
         residual_knm,drift_knm,slew_knm,token,r_norm,delta_norm,sigma_norm"
    )?;

    assert_eq!(frames.len(), history.len(), "Volve: frame/history count mismatch");
    let count = frames.len();

    for (idx, (frame, hist)) in frames.iter().zip(history.iter()).enumerate() {
        let r = hist.triple.r;
        let d = hist.triple.delta;
        let s = hist.triple.sigma;
        let rn = if r_max.abs() > 1e-12 { r / r_max } else { 0.0 };
        let dn = if d_max.abs() > 1e-12 { d / d_max } else { 0.0 };
        let sn = if s_max.abs() > 1e-12 { s / s_max } else { 0.0 };

        writeln!(
            out,
            "{idx},{:.2},{:.6},{:.6},{:.4},{:.2},{:.4},{:.4},{:.6},{:.6},{:.6},{},{:.6},{:.6},{:.6}",
            frame.depth_m,
            frame.observed_tqa_knm,
            frame.baseline_tqa_knm,
            frame.swob_kn,
            frame.rpm,
            frame.hkld_kn,
            frame.sppa_kpa,
            r,
            d,
            s,
            tok(hist.state),
            rn,
            dn,
            sn,
        )?;
    }

    write_env(trace_dir.join("env_volve.csv"), &env)?;
    Ok(count)
}

fn export_esp(crate_root: &Path, trace_dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let env = AdmissibilityEnvelope::default_esp_rotating();
    let r_max = env.r_max.abs().max(env.r_min.abs());
    let d_max = env.delta_max.abs().max(env.delta_min.abs());
    let s_max = env.sigma_max.abs().max(env.sigma_min.abs());

    let mut rdr = csv::Reader::from_path(crate_root.join("data").join("rotating_real.csv"))?;
    let mut rows: Vec<RawEspRow> = Vec::new();
    for result in rdr.deserialize::<RawEspRow>() {
        rows.push(result?);
    }

    let mut engine = DeterministicDsfb::new(env, GrammarClassifier::new());
    for row in &rows {
        let obs = f64_or(&row.rms_broadband, 0.0);
        let base = f64_or(&row.baseline_rms, obs);
        let step = f64_or(&row.step, 0.0);
        let sample = ResidualSample::new(step, obs - base, 0.0, "rms_broadband");
        engine.ingest_sample(&sample);
    }

    let history = engine.history();
    let mut out = File::create(trace_dir.join("real_esp_trace.csv"))?;
    writeln!(
        out,
        "step_idx,step_in_unit,esp_id,label,rms_broadband,baseline_rms,peak1x,peak2x,\
         residual,drift,slew,token,r_norm,delta_norm,sigma_norm"
    )?;

    assert_eq!(rows.len(), history.len(), "ESP: row/history count mismatch");
    let count = rows.len();

    for (idx, (row, hist)) in rows.iter().zip(history.iter()).enumerate() {
        let obs = f64_or(&row.rms_broadband, 0.0);
        let base = f64_or(&row.baseline_rms, obs);
        let peak1x = f64_or(&row.peak1x, 0.0);
        let peak2x = f64_or(&row.peak2x, 0.0);
        let step = f64_or(&row.step, idx as f64);
        let r = hist.triple.r;
        let d = hist.triple.delta;
        let s = hist.triple.sigma;
        let rn = if r_max.abs() > 1e-12 { r / r_max } else { 0.0 };
        let dn = if d_max.abs() > 1e-12 { d / d_max } else { 0.0 };
        let sn = if s_max.abs() > 1e-12 { s / s_max } else { 0.0 };

        writeln!(
            out,
            "{idx},{:.0},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:.6},{:.6}",
            step,
            row.esp_id.trim(),
            row.label.trim(),
            obs,
            base,
            peak1x,
            peak2x,
            r,
            d,
            s,
            tok(hist.state),
            rn,
            dn,
            sn,
        )?;
    }

    write_env(trace_dir.join("env_esp.csv"), &env)?;
    Ok(count)
}

fn write_env(path: PathBuf, env: &AdmissibilityEnvelope) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = File::create(path)?;
    writeln!(out, "param,value")?;
    writeln!(out, "r_min,{}", env.r_min)?;
    writeln!(out, "r_max,{}", env.r_max)?;
    writeln!(out, "delta_min,{}", env.delta_min)?;
    writeln!(out, "delta_max,{}", env.delta_max)?;
    writeln!(out, "sigma_min,{}", env.sigma_min)?;
    writeln!(out, "sigma_max,{}", env.sigma_max)?;
    writeln!(out, "grazing_band,{}", env.grazing_band)?;
    Ok(())
}
