//! SQLShare text-only adapter.
//!
//! The 2015 SQLShare SIGMOD data release used to ship a CSV with per-query
//! runtime and submission-time columns (see `QueriesWithPlan.csv` /
//! `sdssquerieswithplan.csv` in the 2015 reproducibility repository). That
//! richer release was hosted on the S3 bucket `shrquerylogs` at
//! `s3-us-west-2.amazonaws.com`, which was decommissioned; the bucket itself
//! no longer exists (verified 2026-04: `NoSuchBucket` response). The
//! remaining public artefact is the UW eScience `sqlshare_data_release1.zip`
//! bundle, whose top-level `queries.txt` contains raw SQL query texts
//! separated by 40-underscore dividers — no `user_id`, no `runtime_seconds`,
//! no `submitted_at`.
//!
//! This adapter accepts that remaining artefact honestly: it reads the
//! `queries.txt` format, normalises each query into a skeleton (literals and
//! digits replaced with `?`, whitespace collapsed, lower-cased), and emits
//! **only** the `WorkloadPhase` residual class, with Jensen-Shannon divergence
//! computed over **ordinal-position buckets** rather than wall-clock buckets.
//!
//! This is not a temporal analysis. The `t` axis on the emitted residual
//! samples is ordinal-bucket-index (multiplied by the bucket size for
//! plot-axis consistency), and every stream this adapter produces is tagged
//! `sqlshare-text@<file>` so downstream reports cannot confuse it with a
//! wall-clock-indexed SQLShare run. The emitted channel id is
//! `ord[START-END]`, using the ordinal range covered by the bucket.
//!
//! The `PlanRegression`, `Cardinality`, `Contention`, and `CacheIo` classes
//! are all absent: the public release does not carry the fields required to
//! construct them, and fabricating those fields would be a category error.
//! That limitation is documented in §6 of the paper (when colocated) and
//! cited in the README under the Datasets table.

use super::DatasetAdapter;
use crate::residual::{workload_phase, ResidualStream};
use anyhow::{Context, Result};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::path::Path;

pub struct SqlShareText;

/// Ordinal bucket size: number of consecutive queries per histogram window.
/// Chosen so that an ~11 k-query corpus yields ~55 buckets (enough for
/// phase-shift resolution without over-fragmenting the histograms).
const BUCKET_SIZE: usize = 200;

/// The divider line used in `sqlshare_data_release1.zip/queries.txt` between
/// successive query texts. The release ships this as exactly 40 underscores.
const QUERY_DIVIDER: &str = "________________________________________";

fn skeleton(q: &str) -> String {
    let mut out = String::with_capacity(q.len());
    let mut in_str = false;
    let mut prev_ws = false;
    for c in q.chars() {
        if c == '\'' || c == '"' {
            in_str = !in_str;
            out.push('?');
            continue;
        }
        if in_str {
            continue;
        }
        if c.is_ascii_digit() {
            out.push('?');
            prev_ws = false;
            continue;
        }
        if c.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
            continue;
        }
        prev_ws = false;
        for lc in c.to_lowercase() {
            out.push(lc);
        }
    }
    out.trim().to_string()
}

/// Split a `queries.txt`-shaped byte string into its component SQL query
/// texts, preserving file order. Empty queries (from leading/trailing or
/// repeated dividers) are dropped.
fn split_queries(content: &str) -> Vec<&str> {
    content
        .split(QUERY_DIVIDER)
        .map(|q| q.trim())
        .filter(|q| !q.is_empty())
        .collect()
}

impl DatasetAdapter for SqlShareText {
    fn name(&self) -> &'static str {
        "sqlshare-text"
    }

    fn load(&self, path: &Path) -> Result<ResidualStream> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading sqlshare queries.txt at {}", path.display()))?;
        let queries = split_queries(&content);

        let mut stream = ResidualStream::new(format!(
            "sqlshare-text@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ));

        // Bucket consecutive queries by ordinal position; emit one
        // WorkloadPhase residual per bucket boundary using the JS divergence
        // between the bucket's skeleton-histogram and the previous bucket's.
        let mut prev_histo: Option<HashMap<String, u64>> = None;
        let mut current_histo: HashMap<String, u64> = HashMap::new();
        let mut in_bucket: usize = 0;
        let mut bucket_index: usize = 0;

        for q in &queries {
            let sk = skeleton(q);
            *current_histo.entry(sk).or_insert(0) += 1;
            in_bucket += 1;
            if in_bucket == BUCKET_SIZE {
                let start = bucket_index * BUCKET_SIZE;
                let end = start + in_bucket - 1;
                if let Some(prev) = &prev_histo {
                    let d = workload_phase::js_divergence(prev, &current_histo);
                    workload_phase::push_jsd(
                        &mut stream,
                        (bucket_index * BUCKET_SIZE) as f64,
                        &format!("ord[{start}-{end}]"),
                        d,
                    );
                }
                prev_histo = Some(std::mem::take(&mut current_histo));
                in_bucket = 0;
                bucket_index += 1;
            }
        }
        // The partial trailing bucket is intentionally dropped: its
        // sample count is not comparable to a full bucket's, and emitting
        // a JSD against an unequal-sample-size histogram would distort
        // the residual. This matches the full-bucket-only convention the
        // other adapters use for their trailing partials.

        stream.sort();
        Ok(stream)
    }

    fn exemplar(&self, seed: u64) -> ResidualStream {
        // Deterministic structural exemplar: two stable phases plus a
        // phase shift at ordinal bucket 15. The exemplar's `source` tag
        // carries the `-exemplar-seed{N}` marker required by the adapter
        // module-level design rule, so a downstream report cannot
        // mislabel it as real data.
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut stream = ResidualStream::new(format!("sqlshare-text-exemplar-seed{seed}"));
        for b in 1..30 {
            let start = b * BUCKET_SIZE;
            let end = start + BUCKET_SIZE - 1;
            let jsd = if (15..20).contains(&b) {
                0.38 + rng.gen_range(-0.04..0.04)
            } else {
                0.05 + rng.gen_range(0.0..0.03)
            };
            workload_phase::push_jsd(
                &mut stream,
                (b * BUCKET_SIZE) as f64,
                &format!("ord[{start}-{end}]"),
                jsd,
            );
        }
        stream.sort();
        stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residual::ResidualClass;

    #[test]
    fn splits_underscore_separated_queries() {
        let text = "SELECT 1\n\n\n________________________________________\nSELECT 2\n\n________________________________________\nSELECT 3";
        let qs = split_queries(text);
        assert_eq!(qs, vec!["SELECT 1", "SELECT 2", "SELECT 3"]);
    }

    #[test]
    fn drops_empty_between_divider_runs() {
        let text =
            "________________________________________\n\n________________________________________\nSELECT 1";
        assert_eq!(split_queries(text), vec!["SELECT 1"]);
    }

    #[test]
    fn skeleton_strips_literals_and_digits() {
        let a = skeleton("SELECT * FROM t WHERE id = 123 AND name = 'alice'");
        let b = skeleton("select * from t where id = 999 and name = 'bob'");
        assert_eq!(a, b, "skeletons should match after literal/digit stripping");
        assert!(a.contains("select"));
        assert!(a.contains('?'));
        assert!(!a.contains("alice"));
        assert!(!a.contains("bob"));
    }

    #[test]
    fn emits_only_workload_phase_class() {
        // Build a tiny synthetic queries.txt that fills three buckets.
        let mut text = String::new();
        for i in 0..(BUCKET_SIZE * 3) {
            let q = if i < BUCKET_SIZE {
                "select count(*) from t"
            } else if i < BUCKET_SIZE * 2 {
                "select count(*) from u"
            } else {
                "select avg(x) from v group by y"
            };
            text.push_str(q);
            text.push_str("\n________________________________________\n");
        }
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &text).unwrap();
        let stream = SqlShareText.load(tmp.path()).unwrap();
        assert!(stream.source.starts_with("sqlshare-text@"));
        for s in stream.samples.iter() {
            assert_eq!(
                s.class,
                ResidualClass::WorkloadPhase,
                "text-only mode must emit only WorkloadPhase residuals"
            );
        }
        // Two bucket-boundary residuals for three full buckets.
        assert_eq!(stream.samples.iter().count(), 2);
    }

    #[test]
    fn fingerprint_is_deterministic_across_runs() {
        let mut text = String::new();
        for i in 0..(BUCKET_SIZE * 4) {
            let q = if i % 2 == 0 {
                "select * from t where x = 1"
            } else {
                "select a, b from u join v on u.id = v.id"
            };
            text.push_str(q);
            text.push_str("\n________________________________________\n");
        }
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &text).unwrap();
        let s1 = SqlShareText.load(tmp.path()).unwrap();
        let s2 = SqlShareText.load(tmp.path()).unwrap();
        assert_eq!(
            s1.fingerprint(),
            s2.fingerprint(),
            "text-only SQLShare stream must be bytewise-deterministic across reads"
        );
    }
}
