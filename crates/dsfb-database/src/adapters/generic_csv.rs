//! Generic CSV adapter — a single-domain worked example of applying
//! `dsfb-database`'s motif grammar to a residual stream that was not
//! captured from a SQL engine.
//!
//! ## What this adapter does
//!
//! Read an operator-supplied CSV with a timestamp column and a numeric
//! value column (and optionally a channel column), construct a residual
//! stream via the *same* rolling-baseline rule as the PostgreSQL
//! `pg_stat_statements` adapter (see [`crate::adapters::postgres`]), and
//! hand the resulting stream to the motif grammar.
//!
//! ## What this adapter does NOT do
//!
//! It does **not** validate that the operator-supplied grammar is
//! appropriate for the input signal, nor does it claim the five-motif
//! vocabulary has any universal meaning outside SQL telemetry. This
//! adapter is a **worked example** that lets an operator exercise the
//! deterministic machinery on their own residuals; it is not a
//! generalisation claim. See the pinned non-claim in
//! [`crate::non_claims`] that references this adapter by name.
//!
//! ## CSV contract
//!
//! Minimum: one timestamp column, one value column. The adapter
//! auto-detects both:
//!
//!  * Timestamp column — first column whose header contains any of
//!    `{"t", "time", "timestamp", "ts"}` (case-insensitive) or whose
//!    first data row parses as an `f64`. Operators can override with
//!    `--time-col <name>`.
//!  * Value column — first numeric column that is not the timestamp and
//!    whose header is not recognisably a key (`id`, `key`, `uuid`,
//!    `hash`). Operators can override with `--value-col <name>`.
//!  * Channel column — optional. If any column is named `channel`,
//!    `qclass`, `group`, or `series` (case-insensitive), the adapter
//!    emits one residual per (timestamp, channel) row; otherwise the
//!    channel defaults to `generic`.
//!
//! ## Residual construction
//!
//! The adapter emits every row's `value` as a `ResidualClass::PlanRegression`
//! residual in the dimensionless form `(value − baseline) / max(|baseline|, ε)`,
//! where `baseline` is the mean of the first `BASELINE_WINDOW = 3`
//! values on that channel. This is the *same* normalisation the
//! PostgreSQL adapter performs; it keeps the `drift_threshold` in
//! `spec/motifs.yaml` interpretable as a dimensionless fraction.
//!
//! `--pre-residualized` skips the baseline subtraction and emits
//! `value` verbatim. Use this when the operator's pipeline already
//! produces `(actual − expected)` residuals.

use crate::residual::{plan_regression, ResidualClass, ResidualSample, ResidualStream};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Same baseline size as [`crate::adapters::postgres::BASELINE_WINDOW`].
const BASELINE_WINDOW: usize = 3;

/// Upper bound on rows the adapter reads. Matches the pg_stat_statements
/// adapter's `MAX_PGSS_ROWS` order of magnitude so behaviour is uniform.
const MAX_ROWS: usize = 100_000_000;

/// Options for the generic CSV loader. Empty for "auto-detect everything".
#[derive(Debug, Clone, Default)]
pub struct GenericCsvOptions {
    pub time_col: Option<String>,
    pub value_col: Option<String>,
    pub channel_col: Option<String>,
    pub pre_residualized: bool,
}

/// Load `path` as a generic CSV and produce a typed residual stream.
///
/// Errors if the file is missing, has fewer than two rows, the requested
/// columns are absent, or auto-detection cannot identify a timestamp
/// column.
pub fn load_generic_csv(path: &Path, opts: &GenericCsvOptions) -> Result<ResidualStream> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening generic csv at {}", path.display()))?;
    let headers: Vec<String> = rdr
        .headers()
        .context("reading CSV headers")?
        .iter()
        .map(str::to_owned)
        .collect();
    if headers.is_empty() {
        return Err(anyhow!(
            "generic csv at {} has no header row",
            path.display()
        ));
    }

    let all_rows: Vec<csv::StringRecord> = rdr
        .records()
        .take(MAX_ROWS)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("parsing generic csv rows")?;
    if all_rows.len() < 2 {
        return Err(anyhow!(
            "generic csv at {} has fewer than 2 data rows; need ≥2 to compute a baseline",
            path.display()
        ));
    }

    let t_idx = pick_time_col(&headers, &all_rows[0], opts.time_col.as_deref())?;
    let v_idx = pick_value_col(&headers, t_idx, opts.value_col.as_deref())?;
    let c_idx = pick_channel_col(&headers, opts.channel_col.as_deref());

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("anonymous.csv");
    let mut stream = ResidualStream::new(format!("generic-csv@{}", filename));

    // Group (t, value) pairs by channel, then emit in sorted (channel, t) order
    // so the residual stream is byte-stable across runs.
    let mut by_channel: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    for rec in &all_rows {
        let Some(t_raw) = rec.get(t_idx) else {
            continue;
        };
        let Some(v_raw) = rec.get(v_idx) else {
            continue;
        };
        let Ok(t) = t_raw.trim().parse::<f64>() else {
            continue;
        };
        let Ok(v) = v_raw.trim().parse::<f64>() else {
            continue;
        };
        if !t.is_finite() || !v.is_finite() {
            continue;
        }
        let channel = c_idx
            .and_then(|i| rec.get(i))
            .map(str::to_owned)
            .unwrap_or_else(|| "generic".to_string());
        by_channel.entry(channel).or_default().push((t, v));
    }

    if by_channel.is_empty() {
        return Err(anyhow!(
            "generic csv at {} produced no parseable (t, value) pairs",
            path.display()
        ));
    }

    let mut channels_sorted: Vec<String> = by_channel.keys().cloned().collect();
    channels_sorted.sort();

    for ch in &channels_sorted {
        let rows = by_channel.get_mut(ch).unwrap();
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        if opts.pre_residualized {
            for (t, r) in rows.iter() {
                stream.push(
                    ResidualSample::new(*t, ResidualClass::PlanRegression, *r)
                        .with_channel(ch.clone()),
                );
            }
        } else {
            if rows.len() <= BASELINE_WINDOW {
                continue;
            }
            let baseline: f64 =
                rows.iter().take(BASELINE_WINDOW).map(|(_, v)| *v).sum::<f64>()
                    / BASELINE_WINDOW as f64;
            debug_assert!(baseline.is_finite(), "finite baseline from finite inputs");
            for (i, (t, v)) in rows.iter().enumerate() {
                if i < BASELINE_WINDOW {
                    continue;
                }
                plan_regression::push_latency(&mut stream, *t, ch, *v, baseline);
            }
        }
    }

    if stream.is_empty() {
        return Err(anyhow!(
            "generic csv at {} produced no residuals (every channel had < {} + 1 rows)",
            path.display(),
            BASELINE_WINDOW
        ));
    }

    stream.sort();
    Ok(stream)
}

fn pick_time_col(
    headers: &[String],
    first_row: &csv::StringRecord,
    override_name: Option<&str>,
) -> Result<usize> {
    if let Some(name) = override_name {
        return find_header(headers, name)
            .ok_or_else(|| anyhow!("--time-col '{}' not found in {:?}", name, headers));
    }
    let tokens = ["t", "time", "timestamp", "ts"];
    for (i, h) in headers.iter().enumerate() {
        let lo = h.to_ascii_lowercase();
        if tokens.iter().any(|tok| lo == *tok || lo.contains(tok)) {
            return Ok(i);
        }
    }
    for (i, cell) in first_row.iter().enumerate() {
        if cell.trim().parse::<f64>().is_ok() {
            return Ok(i);
        }
    }
    Err(anyhow!(
        "could not auto-detect a timestamp column in {:?}; pass --time-col <name>",
        headers
    ))
}

fn pick_value_col(
    headers: &[String],
    t_idx: usize,
    override_name: Option<&str>,
) -> Result<usize> {
    if let Some(name) = override_name {
        return find_header(headers, name)
            .ok_or_else(|| anyhow!("--value-col '{}' not found in {:?}", name, headers));
    }
    let key_tokens = ["id", "key", "uuid", "hash", "channel", "group", "qclass", "series"];
    for (i, h) in headers.iter().enumerate() {
        if i == t_idx {
            continue;
        }
        let lo = h.to_ascii_lowercase();
        if key_tokens.iter().any(|tok| lo == *tok) {
            continue;
        }
        let value_tokens = ["value", "residual", "latency", "metric", "amount", "v", "y"];
        if value_tokens.iter().any(|tok| lo == *tok || lo.contains(tok)) {
            return Ok(i);
        }
    }
    for (i, h) in headers.iter().enumerate() {
        if i == t_idx {
            continue;
        }
        let lo = h.to_ascii_lowercase();
        if key_tokens.iter().any(|tok| lo == *tok) {
            continue;
        }
        return Ok(i);
    }
    Err(anyhow!(
        "could not auto-detect a value column in {:?}; pass --value-col <name>",
        headers
    ))
}

fn pick_channel_col(headers: &[String], override_name: Option<&str>) -> Option<usize> {
    if let Some(name) = override_name {
        return find_header(headers, name);
    }
    let tokens = ["channel", "qclass", "group", "series"];
    for (i, h) in headers.iter().enumerate() {
        let lo = h.to_ascii_lowercase();
        if tokens.iter().any(|tok| lo == *tok) {
            return Some(i);
        }
    }
    None
}

fn find_header(headers: &[String], name: &str) -> Option<usize> {
    headers.iter().position(|h| h.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_csv(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::Builder::new()
            .suffix(".csv")
            .tempfile()
            .expect("tempfile");
        f.write_all(content.as_bytes()).expect("write");
        f
    }

    #[test]
    fn autodetects_time_value_single_channel() {
        let csv = "t,value\n0,1.0\n1,1.0\n2,1.0\n3,1.5\n4,2.0\n5,2.5\n";
        let f = tmp_csv(csv);
        let s = load_generic_csv(f.path(), &GenericCsvOptions::default()).expect("load");
        assert_eq!(s.len(), 3);
        assert!(s
            .samples
            .iter()
            .all(|r| r.class == ResidualClass::PlanRegression));
        assert!(s.samples.iter().all(|r| r.channel.as_deref() == Some("generic")));
    }

    #[test]
    fn uses_channel_column_when_present() {
        let csv = "time,channel,y\n0,a,1\n1,a,1\n2,a,1\n3,a,2\n0,b,2\n1,b,2\n2,b,2\n3,b,3\n";
        let f = tmp_csv(csv);
        let s = load_generic_csv(f.path(), &GenericCsvOptions::default()).expect("load");
        let channels: std::collections::BTreeSet<_> = s
            .samples
            .iter()
            .filter_map(|r| r.channel.clone())
            .collect();
        assert!(channels.contains("a"));
        assert!(channels.contains("b"));
    }

    #[test]
    fn pre_residualized_skips_baseline() {
        let csv = "t,residual\n0,0.1\n1,0.2\n2,0.3\n";
        let f = tmp_csv(csv);
        let s = load_generic_csv(
            f.path(),
            &GenericCsvOptions {
                pre_residualized: true,
                ..Default::default()
            },
        )
        .expect("load");
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn explicit_overrides_are_honoured() {
        let csv = "alpha,beta,gamma\n0,1.0,x\n1,1.0,x\n2,1.0,x\n3,1.5,x\n4,2.0,x\n5,2.5,x\n";
        let f = tmp_csv(csv);
        let s = load_generic_csv(
            f.path(),
            &GenericCsvOptions {
                time_col: Some("alpha".into()),
                value_col: Some("beta".into()),
                channel_col: Some("gamma".into()),
                pre_residualized: false,
            },
        )
        .expect("load");
        assert!(s
            .samples
            .iter()
            .all(|r| r.channel.as_deref() == Some("x")));
    }

    #[test]
    fn rejects_csv_with_one_row() {
        let csv = "t,value\n0,1.0\n";
        let f = tmp_csv(csv);
        let err = load_generic_csv(f.path(), &GenericCsvOptions::default()).unwrap_err();
        assert!(err.to_string().contains("fewer than 2"));
    }
}
