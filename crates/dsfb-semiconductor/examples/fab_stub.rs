use dsfb_semiconductor::input::residual_stream::ResidualSample;
use dsfb_semiconductor::interface::{DSFBObserver, FabDataSource, ReadOnlyDsfbObserver};

struct ToolResidualTap {
    tool_name: &'static str,
    residuals: Vec<ResidualSample>,
}

impl FabDataSource for ToolResidualTap {
    fn residual_stream(&self) -> Vec<ResidualSample> {
        self.residuals.clone()
    }
}

fn main() {
    let source = ToolResidualTap {
        tool_name: "etch-chamber-12",
        residuals: vec![
            ResidualSample {
                timestamp: 1.0,
                feature_id: "S059".into(),
                value: 0.42,
            },
            ResidualSample {
                timestamp: 2.0,
                feature_id: "S059".into(),
                value: 0.87,
            },
            ResidualSample {
                timestamp: 3.0,
                feature_id: "S059".into(),
                value: 1.31,
            },
        ],
    };

    let observer = ReadOnlyDsfbObserver::new();
    for sample in source.residual_stream() {
        observer.ingest(&sample);
    }

    println!("tool={}", source.tool_name);
    for decision in observer.output() {
        println!("timestamp={:.0} decision={}", decision.timestamp, decision.decision);
    }
}
