use dsfb_oil_gas::{
    generate_all_figures, load_pipeline_csv, AdmissibilityEnvelope, DeterministicDsfb,
    GrammarClassifier,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("figures") | Some("generate-figures") => {
            let result = generate_all_figures()?;
            println!(
                "Generated {} figures in {}",
                result.figure_count,
                result.output_dir.display()
            );
            return Ok(());
        }
        Some(path) => run_pipeline_demo(path),
        None => run_pipeline_demo("data/pipeline_synthetic.csv"),
    }
}

fn run_pipeline_demo(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let frames = load_pipeline_csv(csv_path)?;
    let mut engine = DeterministicDsfb::new(
        AdmissibilityEnvelope::default_pipeline(),
        GrammarClassifier::default(),
    );

    for frame in frames {
        let _ = engine.ingest(frame);
    }

    println!("Detected {} DSFB events", engine.events().len());
    for event in engine.events().iter().take(10) {
        println!("{:?}", event);
    }
    Ok(())
}
