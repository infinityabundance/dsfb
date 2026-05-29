//! Operator-report claim audit (Wave-2 P76).
//!
//! `NoNarrativeHallucinationGateV1` already proves every *narrative sentence* is anchored to a sealed
//! evidence object. This audit extends that discipline to the **whole operator report**: every episode row
//! in `operator_report.html` must anchor to a real fused episode in the analysis result, and the report's
//! headline counts must match the result — so the report can present *nothing* that the evidence does not
//! contain (no fabricated rows, no inflated counts).

use std::fs;

use dsfb_chemical_engineering_edge::cli::synthetic_suite;
use dsfb_chemical_engineering_edge::report::write_operator_report_html;
use dsfb_chemical_engineering_edge::{analyze, PipelineConfig};

#[test]
fn operator_report_rows_are_all_anchored_to_real_episodes() {
    let cfg = PipelineConfig::default();
    let tmp = std::env::temp_dir().join(format!("dsfb_opreport_audit_{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();

    for d in synthetic_suite() {
        let res = analyze(&d.name, &d.kind, &d.matrix, d.n_base, cfg);
        let path = tmp.join(format!("{}.html", d.name));
        write_operator_report_html(&path, &d.name, &res).expect("write operator report");
        let html = fs::read_to_string(&path).unwrap();

        // Each episode row starts with `<tr><td>` (the header row starts with `<tr><th>`), so counting
        // `<tr><td>` is exactly the number of episode rows the report presents.
        let report_rows = html.matches("<tr><td>").count();
        assert_eq!(
            report_rows,
            res.fused_episodes.len(),
            "{}: operator report shows {} episode rows but the analysis has {} fused episodes \
             (a row is either hallucinated or dropped)",
            d.name,
            report_rows,
            res.fused_episodes.len()
        );

        // The headline fused-episode count printed in the prose must match the metric (no inflated count).
        let claimed = format!("<b>{}</b> fused episodes", res.metrics.fused_episodes);
        assert!(
            html.contains(&claimed),
            "{}: operator report headline must state {} fused episodes",
            d.name,
            res.metrics.fused_episodes
        );

        // The claim-boundary banner (Wave-1 A3) must be present so the report is never read as a diagnosis.
        assert!(
            html.contains("CANDIDATE"),
            "{}: operator report must carry the claim-boundary banner",
            d.name
        );
        assert!(
            html.contains("NO CONTROL OR SAFETY"),
            "{}: operator report must state the no-control / no-safety-authority boundary",
            d.name
        );
    }
    let _ = fs::remove_dir_all(&tmp);
}
