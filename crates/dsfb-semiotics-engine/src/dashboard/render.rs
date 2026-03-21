//! ASCII rendering helpers for deterministic dashboard replay frames.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use super::types::{DashboardReplayEvent, DashboardReplayStream, DASHBOARD_EVENT_SCHEMA_VERSION};

/// Replay-session metadata shown in the dashboard top bar.
#[derive(Clone, Debug)]
pub(crate) struct ReplayFrameChrome<'a> {
    pub replay_mode_label: &'a str,
    pub source_label: &'a str,
    pub status_label: &'a str,
    pub playback_speed: f64,
    pub current_time: f64,
}

pub(crate) fn render_frame_ascii(
    stream: &DashboardReplayStream,
    event: &DashboardReplayEvent,
    width: u16,
    height: u16,
    chrome: &ReplayFrameChrome<'_>,
) -> String {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Min(5),
        ])
        .split(area);

    let top_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(vertical[0]);

    let header = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(vertical[1]);
    let metrics = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(vertical[2]);
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(vertical[3]);

    Paragraph::new(format!(
        "{}\nstatus={}",
        chrome.replay_mode_label, chrome.status_label
    ))
    .block(Block::default().title("Replay Mode").borders(Borders::ALL))
    .wrap(Wrap { trim: true })
    .render(top_bar[0], &mut buffer);

    Paragraph::new(format!(
        "source={}\nscenario={}",
        chrome.source_label, stream.scenario_id
    ))
    .block(Block::default().title("Source").borders(Borders::ALL))
    .wrap(Wrap { trim: true })
    .render(top_bar[1], &mut buffer);

    Paragraph::new(format!("replay_time={:.3}", chrome.current_time))
        .block(Block::default().title("Replay Time").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .render(top_bar[2], &mut buffer);

    Paragraph::new(format!("speed={:.2}x", chrome.playback_speed))
        .block(
            Block::default()
                .title("Playback Speed")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true })
        .render(top_bar[3], &mut buffer);

    Paragraph::new(format!(
        "scenario={} frame={}/{}",
        stream.scenario_id,
        event.frame_index + 1,
        event.total_frames
    ))
    .block(
        Block::default()
            .title("Scenario / Stream")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(header[0], &mut buffer);

    Paragraph::new(format!(
        "syntax={}\ngrammar={}\ntrust={:.3}\nsemantics={}",
        event.syntax_label, event.grammar_state, event.trust_scalar, event.semantic_disposition
    ))
    .block(
        Block::default()
            .title("Headline State")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(header[1], &mut buffer);

    Paragraph::new(format!(
        "candidates={}\nselected={}",
        event.semantic_candidates, event.selected_heuristics
    ))
    .block(
        Block::default()
            .title("Semantic Candidates")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(header[2], &mut buffer);

    Paragraph::new(format!(
        "t={:.3}\nstep={}\nresidual={:.6}",
        event.time, event.step, event.residual_norm
    ))
    .block(
        Block::default()
            .title("Residual Norm")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(metrics[0], &mut buffer);

    Paragraph::new(format!(
        "drift={:.6}\nslew={:.6}\nmargin={:.6}\n{}",
        event.drift_norm, event.slew_norm, event.grammar_margin, event.grammar_reason_text
    ))
    .block(Block::default().title("Drift / Slew").borders(Borders::ALL))
    .wrap(Wrap { trim: true })
    .render(metrics[1], &mut buffer);

    Paragraph::new(format!(
        "p1={:.4}\np2={:.4}\np3={:.4}",
        event.projection_1, event.projection_2, event.projection_3
    ))
    .block(
        Block::default()
            .title("Projected Sign")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(metrics[2], &mut buffer);

    Paragraph::new(format!(
        "alarms={}\naudit={}",
        event.comparator_alarms, event.admissibility_audit
    ))
    .block(
        Block::default()
            .title("Comparators / Admissibility")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(metrics[3], &mut buffer);

    Paragraph::new(format!(
        "input_mode={}\nengine_version={}\nbank_version={}",
        stream.input_mode, stream.engine_version, stream.bank_version
    ))
    .block(Block::default().title("Run Metadata").borders(Borders::ALL))
    .wrap(Wrap { trim: true })
    .render(middle[0], &mut buffer);

    let markers = if event.event_markers.is_empty() {
        "none".to_string()
    } else {
        event.event_markers.join(" | ")
    };
    Paragraph::new(format!(
        "scenario_title={}\nmarkers={}\nlog={}",
        event.scenario_title, markers, event.event_log
    ))
    .block(
        Block::default()
            .title("Event / Transition Log")
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: true })
    .render(middle[1], &mut buffer);

    let sign_timeline = build_sign_timeline(stream, event);
    Paragraph::new(sign_timeline)
        .block(
            Block::default()
                .title("Sign Timeline")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true })
        .render(middle[2], &mut buffer);

    Paragraph::new(format!(
        "dashboard_schema={}\nThis ratatui replay consumes engine/evaluation events only; it does not recompute residual, syntax, grammar, semantics, or comparator logic in the UI layer.",
        DASHBOARD_EVENT_SCHEMA_VERSION
    ))
    .block(Block::default().title("Replay Note").borders(Borders::ALL))
    .wrap(Wrap { trim: true })
    .render(vertical[4], &mut buffer);

    buffer_to_string(&buffer)
}

fn build_sign_timeline(stream: &DashboardReplayStream, event: &DashboardReplayEvent) -> String {
    let upto = event.frame_index.min(stream.events.len().saturating_sub(1));
    stream.events[..=upto]
        .iter()
        .map(|item| {
            format!(
                "{:.2}/{:.2}/{:.2}",
                item.projection_1, item.projection_2, item.projection_3
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut lines = Vec::with_capacity(area.height as usize);
    for y in 0..area.height {
        let mut line = String::new();
        for x in 0..area.width {
            line.push_str(buffer[(x, y)].symbol());
        }
        lines.push(line.trim_end().to_string());
    }
    lines.join("\n")
}
