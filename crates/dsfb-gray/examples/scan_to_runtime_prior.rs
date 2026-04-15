use dsfb_gray::{
    derive_static_priors_from_scan, scan_crate_source, DsfbObserver, ObserverConfig,
    ResidualSample, ResidualSource,
};
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let Some(root) = env::args_os().nth(1) else {
        eprintln!("usage: cargo run --example scan_to_runtime_prior -- <crate-source-dir>");
        process::exit(2);
    };
    let root = PathBuf::from(root);

    let report = match scan_crate_source(&root) {
        Ok(report) => report,
        Err(err) => {
            eprintln!("scan failed: {err}");
            process::exit(1);
        }
    };

    let priors = derive_static_priors_from_scan(&report);
    let config = ObserverConfig::fast_response().with_static_priors(priors);
    let mut observer = DsfbObserver::new(ResidualSource::Latency, &config);

    let sample = ResidualSample {
        value: 12.0,
        baseline: 10.0,
        timestamp_ns: 1_000_000_000,
        source: ResidualSource::Latency,
    };
    let result = observer.observe(&sample);
    println!(
        "audit=canonical-broad grammar={:?} reason={:?} prior={:?}",
        result.grammar_state,
        result.reason_evidence.reason_code,
        result.reason_evidence.applied_prior
    );
}
