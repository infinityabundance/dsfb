use crate::error::{DsfbSemiconductorError, Result};
use plotters::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const ARCHITECTURE_WIDTH: u32 = 1600;
const ARCHITECTURE_HEIGHT: u32 = 900;

pub const DSFB_NON_INTRUSIVE_ARCHITECTURE_PNG: &str = "dsfb_non_intrusive_architecture.png";
pub const DSFB_NON_INTRUSIVE_ARCHITECTURE_SVG: &str = "dsfb_non_intrusive_architecture.svg";
pub const NON_INTRUSIVE_INTERFACE_SPEC: &str = "non_intrusive_interface_spec.md";
pub const DSFB_LAYER_ORDER: [&str; 6] = [
    "Residual",
    "Sign",
    "Syntax",
    "Grammar",
    "Semantics",
    "Policy",
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum DsfbAdvisoryState {
    Silent,
    Watch,
    Review,
    Escalate,
}

impl DsfbAdvisoryState {
    pub fn as_lowercase(self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Watch => "watch",
            Self::Review => "review",
            Self::Escalate => "escalate",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpstreamAlarmSnapshot {
    pub ewma_alarm: bool,
    pub spc_alarm: bool,
    pub threshold_alarm: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DsfbObserverInput {
    pub run_index: usize,
    pub timestamp: String,
    pub residuals: Vec<f64>,
    pub upstream_alarms: UpstreamAlarmSnapshot,
    pub metadata_pairs: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DsfbAdvisoryOutput {
    pub run_index: usize,
    pub timestamp: String,
    pub advisory_state: DsfbAdvisoryState,
    pub layer_order: Vec<String>,
    pub advisory_labels: Vec<String>,
    pub advisory_note: String,
    pub fail_safe_isolation_note: String,
}

pub trait NonIntrusiveDsfbObserver {
    fn observe(&self, observations: &[DsfbObserverInput]) -> Vec<DsfbAdvisoryOutput>;

    fn integration_mode(&self) -> &'static str {
        "read_only_side_channel"
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicReplayObserver;

impl NonIntrusiveDsfbObserver for DeterministicReplayObserver {
    fn observe(&self, observations: &[DsfbObserverInput]) -> Vec<DsfbAdvisoryOutput> {
        observations
            .iter()
            .map(|observation| {
                let residual_energy = observation
                    .residuals
                    .iter()
                    .map(|value| value * value)
                    .sum::<f64>();
                let alarm_count = observation.upstream_alarms.ewma_alarm as usize
                    + observation.upstream_alarms.spc_alarm as usize
                    + observation.upstream_alarms.threshold_alarm as usize;
                let advisory_state = if alarm_count >= 2 || residual_energy >= 9.0 {
                    DsfbAdvisoryState::Escalate
                } else if alarm_count == 1 || residual_energy >= 4.0 {
                    DsfbAdvisoryState::Review
                } else if residual_energy > 0.0 {
                    DsfbAdvisoryState::Watch
                } else {
                    DsfbAdvisoryState::Silent
                };
                let advisory_labels = if advisory_state == DsfbAdvisoryState::Silent {
                    vec!["admissible_residual_context".into()]
                } else {
                    vec![
                        format!("upstream_alarm_count={alarm_count}"),
                        format!("residual_energy={residual_energy:.3}"),
                    ]
                };
                DsfbAdvisoryOutput {
                    run_index: observation.run_index,
                    timestamp: observation.timestamp.clone(),
                    advisory_state,
                    layer_order: DSFB_LAYER_ORDER
                        .iter()
                        .map(|layer| (*layer).to_string())
                        .collect(),
                    advisory_labels,
                    advisory_note:
                        "advisory-only interpretation derived from immutable residual observations"
                            .into(),
                    fail_safe_isolation_note:
                        "observer failure cannot alter upstream monitoring behavior".into(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NonIntrusiveArtifacts {
    pub architecture_png_path: PathBuf,
    pub architecture_svg_path: PathBuf,
    pub interface_spec_path: PathBuf,
    pub layer_order: Vec<String>,
    pub integration_mode: String,
}

pub fn materialize_non_intrusive_artifacts(run_dir: &Path) -> Result<NonIntrusiveArtifacts> {
    let figure_dir = run_dir.join("figures");
    fs::create_dir_all(&figure_dir)?;
    let architecture_png_path = figure_dir.join(DSFB_NON_INTRUSIVE_ARCHITECTURE_PNG);
    let architecture_svg_path = figure_dir.join(DSFB_NON_INTRUSIVE_ARCHITECTURE_SVG);
    let interface_spec_path = run_dir.join(NON_INTRUSIVE_INTERFACE_SPEC);

    draw_non_intrusive_architecture_png(&architecture_png_path)?;
    write_non_intrusive_architecture_svg(&architecture_svg_path)?;
    fs::write(
        &interface_spec_path,
        non_intrusive_interface_spec_markdown(),
    )?;

    Ok(NonIntrusiveArtifacts {
        architecture_png_path,
        architecture_svg_path,
        interface_spec_path,
        layer_order: DSFB_LAYER_ORDER
            .iter()
            .map(|layer| (*layer).to_string())
            .collect(),
        integration_mode: "read_only_side_channel".into(),
    })
}

pub fn non_intrusive_interface_spec_markdown() -> String {
    let layer_order = DSFB_LAYER_ORDER.join(" -> ");
    format!(
        "# Non-Intrusive DSFB Interface Specification\n\n\
DSFB is a deterministic, non-intrusive, read-only interpretation layer. It does not replace SPC, EWMA, threshold logic, APC, or controller actuation. Its role is to read upstream residuals and alarms, transform them through a fixed structural stack, and emit advisory interpretations only.\n\n\
## Contract\n\n\
- Integration mode: `read_only_side_channel`\n\
- Fixed layer order: `{layer_order}`\n\
- Inputs are immutable residual observations, upstream alarm snapshots, and optional metadata.\n\
- Outputs are advisory interpretations only: `Silent`, `Watch`, `Review`, or `Escalate`.\n\
- No DSFB API writes back into thresholds, controller gains, recipe parameters, or actuation paths.\n\
- Primary control timing is unchanged because DSFB consumes a side tap of residual/alarm streams.\n\
- Replay is deterministic: identical ordered inputs must yield identical outputs.\n\
- Failure is isolated: if DSFB crashes or is disabled, upstream plant behavior is unchanged.\n\n\
## Input Surface\n\n\
`DsfbObserverInput` contains:\n\
- `run_index`\n\
- `timestamp`\n\
- `residuals`\n\
- `upstream_alarms`\n\
- `metadata_pairs`\n\n\
## Output Surface\n\n\
`DsfbAdvisoryOutput` contains:\n\
- `run_index`\n\
- `timestamp`\n\
- `advisory_state`\n\
- `layer_order`\n\
- `advisory_labels`\n\
- `advisory_note`\n\
- `fail_safe_isolation_note`\n\n\
## Explicit Non-Claims\n\n\
- No control command output exists.\n\
- No threshold-tuning API exists.\n\
- No recipe-write API exists.\n\
- No claim of controller replacement is made.\n\
- No claim of latency benefit is made; the contract is only that DSFB must not add latency to the upstream control loop.\n"
    )
}

fn draw_non_intrusive_architecture_png(output_path: &Path) -> Result<()> {
    let root = BitMapBackend::new(output_path, (ARCHITECTURE_WIDTH, ARCHITECTURE_HEIGHT))
        .into_drawing_area();
    root.fill(&WHITE).map_err(plot_error)?;

    let title_font = ("sans-serif", 34).into_font().style(FontStyle::Bold);
    let body_font = ("sans-serif", 20).into_font();
    let note_font = ("sans-serif", 18).into_font();

    root.draw(&Text::new(
        "DSFB Non-Intrusive Side-Channel Architecture",
        (60, 52),
        title_font,
    ))
    .map_err(plot_error)?;
    root.draw(&Text::new(
        "Primary SPC/EWMA/controller path remains authoritative; DSFB observes residuals and alarms only.",
        (60, 92),
        body_font.clone(),
    ))
    .map_err(plot_error)?;

    draw_box(
        &root,
        (90, 210),
        (350, 330),
        "Process / Tool",
        &[
            "physical process",
            "sensor stream x(k)",
            "no DSFB dependency",
        ],
        RGBColor(235, 235, 235),
        BLACK,
    )?;
    draw_box(
        &root,
        (470, 210),
        (820, 330),
        "SPC / EWMA / Controller",
        &[
            "primary monitoring",
            "thresholds, charts, APC",
            "certified timing unchanged",
        ],
        RGBColor(205, 205, 205),
        BLACK,
    )?;
    draw_box(
        &root,
        (940, 210),
        (1260, 330),
        "Alarm / Actuation Path",
        &[
            "alarms and controller output",
            "upstream authority retained",
            "no DSFB write-back",
        ],
        RGBColor(160, 160, 160),
        WHITE,
    )?;
    draw_box(
        &root,
        (470, 500),
        (870, 700),
        "DSFB Observer Layer",
        &[
            "Residual -> Sign -> Syntax",
            "Grammar -> Semantics -> Policy",
            "advisory interpretation only",
        ],
        RGBColor(245, 245, 245),
        BLACK,
    )?;
    draw_box(
        &root,
        (1020, 520),
        (1450, 680),
        "Operator-Facing Advisory Output",
        &[
            "Silent / Watch / Review / Escalate",
            "typed residual interpretation",
            "fail-safe isolated",
        ],
        RGBColor(215, 215, 215),
        BLACK,
    )?;

    draw_arrow(&root, (350, 270), (470, 270), false, "x(k)")?;
    draw_arrow(&root, (820, 270), (940, 270), false, "alarms / actuation")?;
    draw_arrow(
        &root,
        (650, 330),
        (650, 500),
        true,
        "read-only residual tap",
    )?;
    draw_arrow(&root, (980, 330), (920, 560), true, "read-only alarm tap")?;
    draw_arrow(&root, (870, 600), (1020, 600), false, "advisory only")?;

    root.draw(&Text::new(
        "No arrow returns from DSFB to control. No threshold, recipe, or actuation API exists.",
        (60, 790),
        note_font.clone(),
    ))
    .map_err(plot_error)?;
    root.draw(&Text::new(
        "Deterministic replay: identical ordered inputs yield identical DSFB outputs.",
        (60, 825),
        note_font,
    ))
    .map_err(plot_error)?;

    root.present().map_err(plot_error)?;
    Ok(())
}

fn draw_box(
    root: &DrawingArea<BitMapBackend<'_>, plotters::coord::Shift>,
    top_left: (i32, i32),
    bottom_right: (i32, i32),
    title: &str,
    body: &[&str],
    fill: RGBColor,
    text: RGBColor,
) -> Result<()> {
    root.draw(&Rectangle::new(
        [top_left, bottom_right],
        ShapeStyle::from(&fill).filled(),
    ))
    .map_err(plot_error)?;
    root.draw(&Rectangle::new(
        [top_left, bottom_right],
        ShapeStyle::from(&BLACK).stroke_width(2),
    ))
    .map_err(plot_error)?;
    root.draw(&Text::new(
        title.to_string(),
        (top_left.0 + 18, top_left.1 + 28),
        ("sans-serif", 24)
            .into_font()
            .style(FontStyle::Bold)
            .color(&text),
    ))
    .map_err(plot_error)?;
    for (index, line) in body.iter().enumerate() {
        root.draw(&Text::new(
            (*line).to_string(),
            (top_left.0 + 18, top_left.1 + 62 + (index as i32 * 28)),
            ("sans-serif", 18).into_font().color(&text),
        ))
        .map_err(plot_error)?;
    }
    Ok(())
}

fn draw_arrow(
    root: &DrawingArea<BitMapBackend<'_>, plotters::coord::Shift>,
    start: (i32, i32),
    end: (i32, i32),
    dashed: bool,
    label: &str,
) -> Result<()> {
    if dashed {
        draw_dashed_line(root, start, end, 10)?;
    } else {
        root.draw(&PathElement::new(
            vec![start, end],
            ShapeStyle::from(&BLACK).stroke_width(3),
        ))
        .map_err(plot_error)?;
    }
    let arrow_tip = if end.0 >= start.0 {
        vec![(end.0 - 14, end.1 - 8), end, (end.0 - 14, end.1 + 8)]
    } else if end.1 >= start.1 {
        vec![(end.0 - 8, end.1 - 14), end, (end.0 + 8, end.1 - 14)]
    } else {
        vec![(end.0 - 8, end.1 + 14), end, (end.0 + 8, end.1 + 14)]
    };
    root.draw(&PathElement::new(
        arrow_tip,
        ShapeStyle::from(&BLACK).stroke_width(3),
    ))
    .map_err(plot_error)?;

    let label_x = (start.0 + end.0) / 2;
    let label_y = (start.1 + end.1) / 2 - 14;
    root.draw(&Text::new(
        label.to_string(),
        (label_x, label_y),
        ("sans-serif", 18).into_font(),
    ))
    .map_err(plot_error)?;
    Ok(())
}

fn draw_dashed_line(
    root: &DrawingArea<BitMapBackend<'_>, plotters::coord::Shift>,
    start: (i32, i32),
    end: (i32, i32),
    segments: i32,
) -> Result<()> {
    for segment in 0..segments {
        if segment % 2 == 0 {
            let segment_start = interpolate_point(start, end, segment as f64 / segments as f64);
            let segment_end = interpolate_point(start, end, (segment + 1) as f64 / segments as f64);
            root.draw(&PathElement::new(
                vec![segment_start, segment_end],
                ShapeStyle::from(&BLACK).stroke_width(3),
            ))
            .map_err(plot_error)?;
        }
    }
    Ok(())
}

fn interpolate_point(start: (i32, i32), end: (i32, i32), t: f64) -> (i32, i32) {
    (
        (start.0 as f64 + (end.0 - start.0) as f64 * t).round() as i32,
        (start.1 as f64 + (end.1 - start.1) as f64 * t).round() as i32,
    )
}

fn write_non_intrusive_architecture_svg(output_path: &Path) -> Result<()> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="100%" height="100%" fill="#ffffff"/>
<text x="60" y="52" font-family="sans-serif" font-size="34" font-weight="700" fill="#000000">DSFB Non-Intrusive Side-Channel Architecture</text>
<text x="60" y="92" font-family="sans-serif" font-size="20" fill="#000000">Primary SPC/EWMA/controller path remains authoritative; DSFB observes residuals and alarms only.</text>
<rect x="90" y="210" width="260" height="120" fill="#ebebeb" stroke="#000000" stroke-width="2"/>
<text x="108" y="238" font-family="sans-serif" font-size="24" font-weight="700">Process / Tool</text>
<text x="108" y="272" font-family="sans-serif" font-size="18">physical process</text>
<text x="108" y="300" font-family="sans-serif" font-size="18">sensor stream x(k)</text>
<text x="108" y="328" font-family="sans-serif" font-size="18">no DSFB dependency</text>
<rect x="470" y="210" width="350" height="120" fill="#cdcdcd" stroke="#000000" stroke-width="2"/>
<text x="488" y="238" font-family="sans-serif" font-size="24" font-weight="700">SPC / EWMA / Controller</text>
<text x="488" y="272" font-family="sans-serif" font-size="18">primary monitoring</text>
<text x="488" y="300" font-family="sans-serif" font-size="18">thresholds, charts, APC</text>
<text x="488" y="328" font-family="sans-serif" font-size="18">certified timing unchanged</text>
<rect x="940" y="210" width="320" height="120" fill="#a0a0a0" stroke="#000000" stroke-width="2"/>
<text x="958" y="238" font-family="sans-serif" font-size="24" font-weight="700" fill="#ffffff">Alarm / Actuation Path</text>
<text x="958" y="272" font-family="sans-serif" font-size="18" fill="#ffffff">alarms and controller output</text>
<text x="958" y="300" font-family="sans-serif" font-size="18" fill="#ffffff">upstream authority retained</text>
<text x="958" y="328" font-family="sans-serif" font-size="18" fill="#ffffff">no DSFB write-back</text>
<rect x="470" y="500" width="400" height="200" fill="#f5f5f5" stroke="#000000" stroke-width="2"/>
<text x="488" y="530" font-family="sans-serif" font-size="24" font-weight="700">DSFB Observer Layer</text>
<text x="488" y="566" font-family="sans-serif" font-size="18">Residual -&gt; Sign -&gt; Syntax</text>
<text x="488" y="594" font-family="sans-serif" font-size="18">Grammar -&gt; Semantics -&gt; Policy</text>
<text x="488" y="622" font-family="sans-serif" font-size="18">advisory interpretation only</text>
<rect x="1020" y="520" width="430" height="160" fill="#d7d7d7" stroke="#000000" stroke-width="2"/>
<text x="1038" y="550" font-family="sans-serif" font-size="24" font-weight="700">Operator-Facing Advisory Output</text>
<text x="1038" y="586" font-family="sans-serif" font-size="18">Silent / Watch / Review / Escalate</text>
<text x="1038" y="614" font-family="sans-serif" font-size="18">typed residual interpretation</text>
<text x="1038" y="642" font-family="sans-serif" font-size="18">fail-safe isolated</text>
<line x1="350" y1="270" x2="470" y2="270" stroke="#000000" stroke-width="3"/>
<polyline points="456,262 470,270 456,278" fill="none" stroke="#000000" stroke-width="3"/>
<text x="392" y="248" font-family="sans-serif" font-size="18">x(k)</text>
<line x1="820" y1="270" x2="940" y2="270" stroke="#000000" stroke-width="3"/>
<polyline points="926,262 940,270 926,278" fill="none" stroke="#000000" stroke-width="3"/>
<text x="836" y="248" font-family="sans-serif" font-size="18">alarms / actuation</text>
<line x1="650" y1="330" x2="650" y2="500" stroke="#000000" stroke-width="3" stroke-dasharray="12,8"/>
<polyline points="642,486 650,500 658,486" fill="none" stroke="#000000" stroke-width="3"/>
<text x="674" y="420" font-family="sans-serif" font-size="18">read-only residual tap</text>
<line x1="980" y1="330" x2="920" y2="560" stroke="#000000" stroke-width="3" stroke-dasharray="12,8"/>
<polyline points="912,546 920,560 928,546" fill="none" stroke="#000000" stroke-width="3"/>
<text x="932" y="446" font-family="sans-serif" font-size="18">read-only alarm tap</text>
<line x1="870" y1="600" x2="1020" y2="600" stroke="#000000" stroke-width="3"/>
<polyline points="1006,592 1020,600 1006,608" fill="none" stroke="#000000" stroke-width="3"/>
<text x="912" y="578" font-family="sans-serif" font-size="18">advisory only</text>
<text x="60" y="790" font-family="sans-serif" font-size="18">No arrow returns from DSFB to control. No threshold, recipe, or actuation API exists.</text>
<text x="60" y="825" font-family="sans-serif" font-size="18">Deterministic replay: identical ordered inputs yield identical DSFB outputs.</text>
</svg>
"##,
        width = ARCHITECTURE_WIDTH,
        height = ARCHITECTURE_HEIGHT,
    );
    fs::write(output_path, svg)?;
    Ok(())
}

fn plot_error<E: std::fmt::Display>(err: E) -> DsfbSemiconductorError {
    DsfbSemiconductorError::ExternalCommand(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_replay_observer_is_stable() {
        let observer = DeterministicReplayObserver;
        let input = vec![
            DsfbObserverInput {
                run_index: 7,
                timestamp: "2008-07-19 21:57:00".into(),
                residuals: vec![0.0, 1.0, 2.0],
                upstream_alarms: UpstreamAlarmSnapshot {
                    ewma_alarm: true,
                    spc_alarm: false,
                    threshold_alarm: false,
                },
                metadata_pairs: vec![("tool".into(), "etch".into())],
            },
            DsfbObserverInput {
                run_index: 8,
                timestamp: "2008-07-19 22:02:00".into(),
                residuals: vec![0.0, 0.0, 0.0],
                upstream_alarms: UpstreamAlarmSnapshot {
                    ewma_alarm: false,
                    spc_alarm: false,
                    threshold_alarm: false,
                },
                metadata_pairs: vec![("tool".into(), "etch".into())],
            },
        ];
        let first = observer.observe(&input);
        let second = observer.observe(&input);
        assert_eq!(first, second);
        assert_eq!(
            first[0].layer_order.join(" -> "),
            DSFB_LAYER_ORDER.join(" -> ")
        );
    }

    #[test]
    fn advisory_output_surface_contains_no_feedback_keys() {
        let observer = DeterministicReplayObserver;
        let output = observer.observe(&[DsfbObserverInput {
            run_index: 1,
            timestamp: "2008-01-01 00:00:00".into(),
            residuals: vec![3.0],
            upstream_alarms: UpstreamAlarmSnapshot {
                ewma_alarm: true,
                spc_alarm: true,
                threshold_alarm: false,
            },
            metadata_pairs: vec![],
        }]);
        let serialized = serde_json::to_string(&output[0]).unwrap();
        for forbidden in ["controller", "actuation", "recipe", "threshold_write"] {
            assert!(
                !serialized.contains(forbidden),
                "unexpected feedback surface key {forbidden}"
            );
        }
    }

    #[test]
    fn architecture_artifacts_are_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let first_dir = temp.path().join("first");
        let second_dir = temp.path().join("second");
        fs::create_dir_all(&first_dir).unwrap();
        fs::create_dir_all(&second_dir).unwrap();

        let first = materialize_non_intrusive_artifacts(&first_dir).unwrap();
        let second = materialize_non_intrusive_artifacts(&second_dir).unwrap();

        assert_eq!(
            fs::read(first.architecture_png_path).unwrap(),
            fs::read(second.architecture_png_path).unwrap()
        );
        assert_eq!(
            fs::read_to_string(first.architecture_svg_path).unwrap(),
            fs::read_to_string(second.architecture_svg_path).unwrap()
        );
        assert!(first.interface_spec_path.exists());
    }
}
