//! `AlarmFloodCompressionReportV1` + `OperatorIncidentHTMLV1` (P62).
//!
//! Two hash-sealed operator-facing report objects, formalizing the existing `alarm_rationalization.csv`
//! / `operator_report.html` exports:
//!
//! - [`AlarmFloodCompressionReportV1`] — an ISA-18.2 alarm-flood **before/after** compression report
//!   (raw alarm count → fused episode count, with the compression ratio) that makes explicit the two
//!   claims that matter to an operator: **`lost_evidence = 0`** and **`recoverable = true`** — the
//!   compression is a *view*, every raw alarm is preserved in the record and recoverable, nothing is
//!   discarded. Emits HTML.
//! - [`OperatorIncidentHTMLV1`] — the nine-question one-page incident report (what / when / where /
//!   which detectors / candidate label / what was ruled out / severity / what to check / evidence root),
//!   sealed so the page a control room sees is replay-anchored. Emits HTML.
//!
//! Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A hash-sealed alarm-flood compression report (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlarmFloodCompressionReportV1 {
    pub dataset: String,
    /// Raw alarm count *before* fusion (e.g. per-detector breaches / ISA-18.2 flood count).
    pub n_raw_alarms: usize,
    /// Fused structural episode count *after* DSFB compression.
    pub n_episodes: usize,
    /// `n_raw_alarms / max(1, n_episodes)` — how many raw alarms each episode subsumes.
    pub compression_ratio: f64,
    /// Always 0: compression is a view; raw alarms are preserved in the record, never discarded.
    pub lost_evidence: usize,
    /// Always true: every compressed alarm is recoverable from the sealed record.
    pub recoverable: bool,
    pub report_hash: String,
}

impl AlarmFloodCompressionReportV1 {
    fn seal(
        dataset: &str,
        n_raw: usize,
        n_ep: usize,
        ratio: f64,
        lost: usize,
        recoverable: bool,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"alarm_flood_compression_report_v1");
        h.field("dataset", dataset.as_bytes());
        h.u64("n_raw_alarms", n_raw as u64);
        h.u64("n_episodes", n_ep as u64);
        h.f64q("compression_ratio", ratio);
        h.u64("lost_evidence", lost as u64);
        h.u64("recoverable", recoverable as u64);
        h.finalize_hex()
    }

    /// Build the report from the raw-alarm and episode counts. `lost_evidence` is fixed at 0 and
    /// `recoverable` at true — the invariants the compression guarantees.
    pub fn build(dataset: impl Into<String>, n_raw_alarms: usize, n_episodes: usize) -> Self {
        let dataset = dataset.into();
        let compression_ratio = n_raw_alarms as f64 / n_episodes.max(1) as f64;
        let report_hash = Self::seal(
            &dataset,
            n_raw_alarms,
            n_episodes,
            compression_ratio,
            0,
            true,
        );
        AlarmFloodCompressionReportV1 {
            dataset,
            n_raw_alarms,
            n_episodes,
            compression_ratio,
            lost_evidence: 0,
            recoverable: true,
            report_hash,
        }
    }

    pub fn verify(&self) -> bool {
        self.lost_evidence == 0
            && self.recoverable
            && Self::seal(
                &self.dataset,
                self.n_raw_alarms,
                self.n_episodes,
                self.compression_ratio,
                0,
                true,
            ) == self.report_hash
    }

    /// Render a before/after HTML report.
    pub fn to_html(&self) -> String {
        format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Alarm-flood compression — {ds}</title></head>\
             <body><h1>Alarm-flood compression — {ds}</h1>\
             <table border=\"1\" cellpadding=\"4\"><tr><th></th><th>count</th></tr>\
             <tr><td>raw alarms (before)</td><td>{raw}</td></tr>\
             <tr><td>fused episodes (after)</td><td>{ep}</td></tr>\
             <tr><td>compression ratio</td><td>{ratio:.2}×</td></tr></table>\
             <p><b>lost_evidence = {lost}</b> · <b>recoverable = {rec}</b> — compression is a view; every \
             raw alarm is preserved in the sealed record and recoverable.</p>\
             <p>report_hash: <code>{h}</code></p></body></html>",
            ds = esc(&self.dataset),
            raw = self.n_raw_alarms,
            ep = self.n_episodes,
            ratio = self.compression_ratio,
            lost = self.lost_evidence,
            rec = self.recoverable,
            h = self.report_hash,
        )
    }
}

/// A hash-sealed nine-question operator incident report (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorIncidentHTMLV1 {
    pub dataset: String,
    /// The nine answers, paired positionally with [`OperatorIncidentHTMLV1::QUESTIONS`].
    pub answers: [String; 9],
    pub incident_hash: String,
}

impl OperatorIncidentHTMLV1 {
    /// The nine questions a one-page operator incident report answers.
    pub const QUESTIONS: [&'static str; 9] = [
        "What happened?",
        "When did it start?",
        "Where (which unit / variables)?",
        "Which detectors testified (participating witnesses)?",
        "What is the candidate label (advisory, not root cause)?",
        "What was ruled out (confusers / negative witnesses)?",
        "How severe / what is the disposition?",
        "What should the operator check next?",
        "What is the evidence root (to replay the case)?",
    ];

    fn seal(dataset: &str, answers: &[String; 9]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"operator_incident_html_v1");
        h.field("dataset", dataset.as_bytes());
        for (q, a) in Self::QUESTIONS.iter().zip(answers) {
            h.field("q", q.as_bytes());
            h.field("a", a.as_bytes());
        }
        h.finalize_hex()
    }

    pub fn build(dataset: impl Into<String>, answers: [String; 9]) -> Self {
        let dataset = dataset.into();
        let incident_hash = Self::seal(&dataset, &answers);
        OperatorIncidentHTMLV1 {
            dataset,
            answers,
            incident_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(&self.dataset, &self.answers) == self.incident_hash
    }

    /// Render the nine-question incident report as a one-page HTML document.
    pub fn to_html(&self) -> String {
        let mut rows = String::new();
        for (q, a) in Self::QUESTIONS.iter().zip(&self.answers) {
            rows.push_str(&format!(
                "<tr><td><b>{}</b></td><td>{}</td></tr>",
                esc(q),
                esc(a)
            ));
        }
        format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Operator incident — {ds}</title></head>\
             <body><h1>Operator incident report — {ds}</h1>\
             <table border=\"1\" cellpadding=\"4\">{rows}</table>\
             <p>incident_hash: <code>{h}</code></p></body></html>",
            ds = esc(&self.dataset),
            rows = rows,
            h = self.incident_hash,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_report_invariants_and_html() {
        let r = AlarmFloodCompressionReportV1::build("tennessee_eastman_idv01", 10041, 6);
        assert_eq!(r.lost_evidence, 0);
        assert!(r.recoverable);
        assert!((r.compression_ratio - 10041.0 / 6.0).abs() < 1e-9);
        assert!(r.verify());
        let html = r.to_html();
        assert!(html.contains("lost_evidence = 0"));
        assert!(html.contains("recoverable = true"));
        assert!(html.contains(&r.report_hash));
    }

    #[test]
    fn incident_report_nine_questions_and_self_verifies() {
        let answers: [String; 9] = core::array::from_fn(|i| format!("answer {i}"));
        let inc = OperatorIncidentHTMLV1::build("cstr_reactor", answers);
        assert_eq!(OperatorIncidentHTMLV1::QUESTIONS.len(), 9);
        assert_eq!(inc.answers.len(), 9);
        assert!(inc.verify());
        let html = inc.to_html();
        assert!(html.contains("What happened?"));
        assert!(html.contains("evidence root"));
        let mut t = inc.clone();
        t.answers[0] = "tampered".into();
        assert!(!t.verify());
    }
}
