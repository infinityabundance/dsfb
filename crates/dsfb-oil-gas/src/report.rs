/// DSFB Oil & Gas — Episode Report Generation
///
/// Produces operator-facing text and CSV summaries from episode logs.
/// Outputs are annotation only; they are not alarms, setpoints, or commands.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{Episode, EpisodeSummary, GrammarState};

// ─────────────────────────────────────────────────────────────────────────────
// Text report
// ─────────────────────────────────────────────────────────────────────────────

/// Format a human-readable episode summary report.
pub fn format_summary(summary: &EpisodeSummary) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "=== DSFB Episode Summary: channel='{}' ===\n",
        summary.channel
    ));
    out.push_str(&format!("  Total steps    : {}\n", summary.total_steps));
    out.push_str(&format!("  Total episodes : {}\n", summary.total_episodes));
    out.push_str(&format!(
        "  Episode Collapse Ratio (ECC) : {:.2}\n",
        summary.episode_count_collapse
    ));
    out.push_str(&format!(
        "  Event Density Reduction (EDR): {:.1}%\n",
        summary.event_density_reduction * 100.0
    ));
    out.push_str(&format!(
        "  Non-nominal episodes : {}\n",
        summary.non_nominal_episodes
    ));
    out.push_str("\n  State distribution:\n");

    let mut states: Vec<(GrammarState, usize)> = summary.by_state.iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    states.sort_by_key(|(_, v)| core::cmp::Reverse(*v));

    for (state, count) in states {
        let pct = count as f64 / summary.total_steps.max(1) as f64 * 100.0;
        out.push_str(&format!(
            "    {:3}  {:>5} steps  ({:5.1}%)\n",
            state.token(),
            count,
            pct,
        ));
    }
    out.push_str("\nNOTE: This report is a deterministic annotation of residual structure.\n");
    out.push_str("      It does not represent an alarm, diagnosis, or prediction.\n");
    out
}

/// Format a compact episode table as plain text.
pub fn format_episodes_table(episodes: &[Episode]) -> String {
    let header = format!(
        "{:<6} {:<26} {:>10} {:>10} {:>6} {:>8} {:>8} {:>10}  {}\n",
        "STATE", "CHANNEL", "START_TS", "END_TS", "STEPS", "PEAK_R", "PEAK_DA", "PEAK_SS", "REASON"
    );
    let sep = "-".repeat(110) + "\n";
    let mut out = header + &sep;
    for ep in episodes {
        out.push_str(&format!(
            "{:<6} {:<26} {:>10.2} {:>10.2} {:>6} {:>8.3} {:>8.3} {:>10.3}  {}\n",
            ep.state.token(),
            &ep.channel[..ep.channel.len().min(26)],
            ep.start_ts,
            ep.end_ts,
            ep.step_count,
            ep.peak_r,
            ep.peak_delta,
            ep.peak_sigma,
            &ep.reason.as_str()[..ep.reason.as_str().len().min(60)],
        ));
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Noise compression ratio (NCR)
// ─────────────────────────────────────────────────────────────────────────────

/// Estimate the noise compression ratio.
///
/// NCR = raw_bytes / episode_bytes (with standard encoding).
///
/// Raw sample encoding (32 bytes):
///   timestamp  f64  8 bytes
///   residual   f64  8 bytes
///   expected   f64  8 bytes
///   observed   f64  8 bytes
///
/// Compact episode encoding (40 bytes, 8-byte aligned):
///   state       u8   1 byte
///   drift_sign  i8   1 byte
///   _pad        u16  2 bytes
///   step_count  u32  4 bytes
///   start_ts    f64  8 bytes
///   end_ts      f64  8 bytes
///   peak_r      f32  4 bytes
///   peak_delta  f32  4 bytes
///   peak_sigma  f32  4 bytes
///   _pad2       u32  4 bytes   (align to 40 bytes)
///
/// Total: 1+1+2+4+8+8+4+4+4+4 = 40 bytes per episode.
pub fn noise_compression_ratio(total_steps: usize, total_episodes: usize) -> f64 {
    let raw_bytes = total_steps as f64 * 32.0;
    let ep_bytes  = total_episodes as f64 * 40.0;
    if ep_bytes > 0.0 { raw_bytes / ep_bytes } else { 1.0 }
}

// ─────────────────────────────────────────────────────────────────────────────
// JSON-lines output (no external dependency — uses manual formatting)
// ─────────────────────────────────────────────────────────────────────────────

/// Serialize episodes to newline-delimited JSON for historian ingest.
pub fn episodes_to_jsonl(episodes: &[Episode]) -> String {
    let mut out = String::new();
    for ep in episodes {
        out.push_str(&format!(
            "{{\"state\":\"{}\",\"channel\":\"{}\",\"start_ts\":{:.4},\"end_ts\":{:.4},\
             \"step_count\":{},\"peak_r\":{:.4},\"peak_delta\":{:.4},\"peak_sigma\":{:.4},\
             \"drift_sign\":{:.0},\"reason\":\"{}\"}}\n",
            ep.state.token(),
            ep.channel,
            ep.start_ts, ep.end_ts,
            ep.step_count,
            ep.peak_r, ep.peak_delta, ep.peak_sigma,
            ep.drift_sign,
            ep.reason.as_str().replace('"', "'"),
        ));
    }
    out
}

