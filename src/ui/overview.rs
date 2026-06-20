use std::collections::VecDeque;

use crate::state::AppState;
use crate::types::Severity;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Axis, Block, Chart, GraphType, LineGauge, Paragraph, Sparkline, Wrap},
    Frame,
};

use super::common::*;
use super::theme;

struct KpiCard<'a> {
    label: &'a str,
    value: String,
    detail: String,
    pct: f64,
    thresholded: bool,
    history: &'a VecDeque<(f64, f64)>,
    spark_max: Option<u64>,
    spark_color: Option<ratatui::style::Color>,
}

pub fn overview(frame: &mut Frame, area: Rect, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if area.height < 24 { 6 } else { 7 }),
            Constraint::Length(if area.height < 24 { 7 } else { 8 }),
            Constraint::Min(8),
        ])
        .split(area);

    render_status_strip(frame, sections[0], app);
    render_kpi_row(frame, sections[1], app);
    render_body(frame, sections[2], app);
}

fn render_kpi_row(frame: &mut Frame, area: Rect, app: &AppState) {
    let t = theme::get();
    let chunks = kpi_layout(area);

    let cpu = app.cpu_usage;
    let mem = app.mem_pct();
    let dsk = app.disk.pct as f64;
    let gpu = app.primary_gpu();
    let gpu_usage = gpu.and_then(|gpu| gpu.usage_pct);
    let empty_history = VecDeque::new();

    render_kpi(
        frame,
        chunks[0],
        KpiCard {
            label: "CPU Load",
            value: format!("{:.1}%", cpu),
            detail: format!("Peak {:.1}%", app.max_core_usage()),
            pct: cpu,
            thresholded: true,
            history: &app.cpu_history,
            spark_max: Some(100),
            spark_color: None,
        },
    );
    render_kpi(
        frame,
        chunks[1],
        KpiCard {
            label: "GPU Load",
            value: gpu_usage
                .map(|value| format!("{value:.0}%"))
                .unwrap_or_else(|| String::from("N/A")),
            detail: gpu
                .map(gpu_load_detail)
                .unwrap_or_else(|| String::from("No GPU")),
            pct: gpu_usage.unwrap_or(0.0),
            thresholded: gpu_usage.is_some(),
            history: &app.gpu_usage_history,
            spark_max: Some(100),
            spark_color: Some(t.accent_purple),
        },
    );
    render_kpi(
        frame,
        chunks[2],
        KpiCard {
            label: "Memory Use",
            value: format!("{:.1}%", mem),
            detail: compact_gb_pair(app.mem_used / 1024.0, app.mem_total / 1024.0),
            pct: mem,
            thresholded: true,
            history: &app.mem_history,
            spark_max: Some(100),
            spark_color: None,
        },
    );
    render_kpi(
        frame,
        chunks[3],
        KpiCard {
            label: "Root Disk",
            value: format!("{}%", app.disk.pct),
            detail: compact_gb_pair(app.disk.used_gb, app.disk.total_gb),
            pct: dsk,
            thresholded: true,
            history: &empty_history,
            spark_max: Some(100),
            spark_color: None,
        },
    );
}

fn kpi_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area)
        .to_vec()
}

fn render_status_strip(frame: &mut Frame, area: Rect, app: &AppState) {
    let t = theme::get();
    let health_sev = Severity::from_health(app.health_score as f64);
    let data_color = sample_status_color(app.sample_status());
    let temp = app
        .temp_c
        .map(|value| format!("{value:.0}\u{b0}C"))
        .unwrap_or_else(|| String::from("N/A"));
    let gpu = app
        .primary_gpu()
        .map(|gpu| {
            let usage = gpu
                .usage_pct
                .map(|value| format!("{value:.0}%"))
                .unwrap_or_else(|| String::from("N/A"));
            let temp = gpu
                .temp_c
                .map(|value| format!("{value:.0}\u{b0}C"))
                .unwrap_or_else(|| String::from("N/A"));
            format!("{} {usage} {temp}", gpu.kind)
        })
        .unwrap_or_else(|| String::from("No GPU"));
    let primary_cause = app.root_causes.first();
    let guidance = primary_cause
        .map(|cause| cause.detail.as_str())
        .unwrap_or("Collecting system guidance");

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(30)])
        .split(area);

    let score_lines = vec![
        Line::from(Span::styled(
            format!(
                "{} {}% {}",
                health_sev.symbol(),
                app.health_score,
                severity_label(health_sev)
            ),
            Style::default()
                .fg(severity_color(health_sev))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Data ", Style::default().fg(t.overlay0)),
            Span::styled(
                app.sample_status(),
                Style::default().fg(data_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!(
                "Updated {:.1}s ago",
                app.last_sample_at.elapsed().as_secs_f64()
            ),
            Style::default().fg(t.overlay1),
        )),
    ];
    frame.render_widget(
        Paragraph::new(score_lines)
            .alignment(Alignment::Center)
            .block(panel_block_severity("System health", health_sev)),
        chunks[0],
    );

    let cause_sev = primary_cause
        .map(|cause| cause.severity)
        .unwrap_or(Severity::Ok);
    let detail_lines = vec![
        Line::from(vec![
            Span::styled("Focus  ", Style::default().fg(t.overlay0)),
            Span::styled(
                primary_cause
                    .map(|cause| format!("{} {}", cause.severity.symbol(), cause.title))
                    .unwrap_or_else(|| String::from("○ No dominant pressure")),
                Style::default()
                    .fg(severity_color(cause_sev))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            truncate(guidance, 86),
            Style::default().fg(t.text),
        )),
        Line::from(vec![
            Span::styled("Thermal ", Style::default().fg(t.overlay0)),
            Span::styled(temp, Style::default().fg(t.accent_orange)),
            Span::styled("  |  GPU ", Style::default().fg(t.overlay0)),
            Span::styled(gpu, Style::default().fg(t.accent_purple)),
            Span::styled("  |  Net ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!(
                    "D {}/s U {}/s",
                    format_bytes(app.net_down_bps),
                    format_bytes(app.net_up_bps)
                ),
                Style::default().fg(t.accent_teal),
            ),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(detail_lines)
            .wrap(Wrap { trim: true })
            .block(panel_block("Operator focus")),
        chunks[1],
    );
}

fn render_kpi(frame: &mut Frame, area: Rect, card: KpiCard<'_>) {
    let t = theme::get();
    let severity = if card.thresholded {
        Severity::from_usage(card.pct)
    } else {
        Severity::Neutral
    };
    let block = panel_block_severity(format!(" {} ", card.label), severity);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(val_span_for(&card, severity)),
        Line::from(vec![
            Span::styled(
                format!("{}  ", severity_label(severity)),
                Style::default()
                    .fg(severity_color(severity))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(card.detail.clone(), Style::default().fg(t.overlay0)),
        ]),
    ];

    if inner.height <= 3 {
        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
        return;
    }

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(2)])
        .split(inner);

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), body[0]);

    if card.history.len() >= 2 && body[1].height >= 2 {
        let spark_data: Vec<u64> = card
            .history
            .iter()
            .rev()
            .take(30)
            .rev()
            .map(|(_, v)| v.max(0.0).round() as u64)
            .collect();
        if spark_data.len() >= 2 {
            let max = card
                .spark_max
                .unwrap_or_else(|| spark_data.iter().copied().max().unwrap_or(1).max(1));
            frame.render_widget(
                Sparkline::default().data(&spark_data).max(max).style(
                    Style::default()
                        .fg(card.spark_color.unwrap_or_else(|| severity_color(severity))),
                ),
                body[1],
            );
        }
    }
}

fn val_span_for(card: &KpiCard<'_>, severity: Severity) -> Span<'static> {
    Span::styled(
        format!("{} {}", severity.symbol(), card.value),
        Style::default()
            .fg(severity_color(severity))
            .add_modifier(Modifier::BOLD),
    )
}

fn compact_gb_pair(used_gb: f64, total_gb: f64) -> String {
    if total_gb >= 100.0 {
        format!("{:.0}/{:.0}G", used_gb, total_gb)
    } else {
        format!("{:.1}/{:.1}G", used_gb, total_gb)
    }
}

fn gpu_load_detail(gpu: &crate::types::GpuInfo) -> String {
    match (gpu.frequency_mhz, gpu.max_frequency_mhz) {
        (Some(cur), Some(max)) if max >= 1000 => {
            format!("{} {cur}/{:.2}G", gpu.kind, max as f64 / 1000.0)
        }
        (Some(cur), Some(max)) => format!("{} {cur}/{max}M", gpu.kind),
        (Some(cur), None) => format!("{} {cur}MHz", gpu.kind),
        _ => format!("{} {}", gpu.kind, gpu.driver),
    }
}

fn render_body(frame: &mut Frame, area: Rect, app: &AppState) {
    if area.height < 12 {
        render_alert_block(frame, area, app);
        return;
    }

    if area.width < 110 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);
        render_charts(frame, chunks[0], app);
        render_analysis_panel(frame, chunks[1], app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(52),
            Constraint::Percentage(22),
            Constraint::Percentage(26),
        ])
        .split(area);

    render_charts(frame, chunks[0], app);
    render_resource_bars(frame, chunks[1], app);
    render_alert_block(frame, chunks[2], app);
}

fn render_analysis_panel(frame: &mut Frame, area: Rect, app: &AppState) {
    if area.height < 10 {
        render_alert_block(frame, area, app);
        return;
    }

    let chunks = if area.width < 70 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(3)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area)
    };

    render_resource_bars(frame, chunks[0], app);
    render_alert_block(frame, chunks[1], app);
}

fn render_charts(frame: &mut Frame, area: Rect, app: &AppState) {
    let t = theme::get();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let cpu_avg = moving_average(&app.cpu_history, 5);
    let cpu_peak = app.cpu_history.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let cpu_title = format!("CPU trend  average {:.0}% | peak {:.0}%", cpu_avg, cpu_peak);
    render_chart(
        frame,
        chunks[0],
        &cpu_title,
        &app.cpu_history,
        t.accent_teal,
    );

    let mem_avg = moving_average(&app.mem_history, 5);
    let mem_peak = app.mem_history.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let mem_title = format!(
        "Memory trend  average {:.0}% | peak {:.0}%",
        mem_avg, mem_peak
    );
    render_chart(
        frame,
        chunks[1],
        &mem_title,
        &app.mem_history,
        t.accent_yellow,
    );
}

fn moving_average(data: &VecDeque<(f64, f64)>, window: usize) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let vals: Vec<f64> = data.iter().rev().take(window).map(|(_, v)| *v).collect();
    vals.iter().sum::<f64>() / vals.len() as f64
}

fn render_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    data: &VecDeque<(f64, f64)>,
    color: ratatui::style::Color,
) {
    if data.is_empty() {
        frame.render_widget(
            Paragraph::new("Collecting...")
                .block(chart_block(title, color))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let t = theme::get();
    let points: Vec<(f64, f64)> = data.iter().copied().collect();
    let x_start = data.front().map(|p| p.0).unwrap_or(0.0);
    let x_end = data.back().map(|p| p.0).unwrap_or(1.0).max(x_start + 1.0);

    let line_set = ratatui::widgets::Dataset::default()
        .name("line")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .data(&points);

    let chart = Chart::new(vec![line_set])
        .block(chart_block(title, color))
        .x_axis(
            Axis::default()
                .bounds([x_start, x_end])
                .style(Style::default().fg(t.overlay1)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .labels(vec![
                    Span::styled("0", Style::default().fg(t.overlay1)),
                    Span::styled("50", Style::default().fg(t.overlay1)),
                    Span::styled("100", Style::default().fg(t.overlay1)),
                ])
                .style(Style::default().fg(t.overlay1)),
        );
    frame.render_widget(chart, area);
}

fn chart_block(title: &str, color: ratatui::style::Color) -> Block<'static> {
    panel_block("")
        .title(accessible_chart_title(title, color))
        .border_style(Style::default().fg(color))
}

fn accessible_chart_title(title: &str, color: ratatui::style::Color) -> Line<'static> {
    let t = theme::get();
    let (metric, rest) = title
        .split_once(" trend")
        .map(|(metric, rest)| (metric, rest.trim()))
        .unwrap_or((title, ""));

    let mut spans = vec![Span::styled(
        format!(" {} TREND ", metric.to_uppercase()),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )];

    if let Some((avg, peak)) = parse_average_peak(rest) {
        spans.extend([
            Span::styled(" AVG ", Style::default().fg(t.overlay1)),
            Span::styled(
                avg,
                Style::default().fg(t.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  PEAK ", Style::default().fg(t.overlay1)),
            Span::styled(
                peak,
                Style::default()
                    .fg(t.accent_yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
    } else if !rest.is_empty() {
        spans.push(Span::styled(
            format!(" {}", rest.to_uppercase()),
            Style::default().fg(t.overlay0).add_modifier(Modifier::BOLD),
        ));
    }

    Line::from(spans)
}

fn parse_average_peak(rest: &str) -> Option<(String, String)> {
    let after_average = rest.strip_prefix("average ")?;
    let (avg, after_peak) = after_average.split_once(" | peak ")?;
    Some((avg.to_string(), after_peak.to_string()))
}

fn render_resource_bars(frame: &mut Frame, area: Rect, app: &AppState) {
    let t = theme::get();
    let block = panel_block("Pressure meters");
    let inner = block.inner(area).inner(&Margin {
        horizontal: 2,
        vertical: 0,
    });
    frame.render_widget(block, area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_pressure_meter(frame, rows[0], "CPU", app.cpu_usage, t.accent_teal);
    render_pressure_meter(frame, rows[1], "Memory", app.mem_pct(), t.accent_yellow);
    render_pressure_meter(frame, rows[2], "Swap", app.swap_pct(), t.accent_purple);
    render_pressure_meter(
        frame,
        rows[3],
        "Root disk",
        app.disk.pct as f64,
        t.accent_green,
    );
}

fn render_pressure_meter(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: f64,
    color: ratatui::style::Color,
) {
    use ratatui::symbols;
    let t = theme::get();
    let sev = Severity::from_usage(value);
    if area.width < 34 {
        let compact = Line::from(vec![
            Span::styled(
                format!("{:<9}", truncate(label, 9)),
                Style::default().fg(t.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{value:>5.1}% "),
                Style::default()
                    .fg(severity_color(sev))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                severity_label(sev),
                Style::default().fg(severity_color(sev)),
            ),
        ]);
        frame.render_widget(Paragraph::new(compact), area);
        return;
    }

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(Span::styled(
            label.to_string(),
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        )),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("{value:>5.1}%"),
            Style::default()
                .fg(severity_color(sev))
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Right),
        columns[1],
    );

    let gauge = LineGauge::default()
        .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .line_set(symbols::line::THICK)
        .label(Span::raw(""))
        .ratio((value / 100.0).clamp(0.0, 1.0));
    frame.render_widget(gauge, columns[2]);
    frame.render_widget(
        Paragraph::new(Span::styled(
            severity_label(sev),
            Style::default().fg(severity_color(sev)),
        ))
        .alignment(Alignment::Right),
        columns[3],
    );
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Ok => "OK",
        Severity::Warn => "WARN",
        Severity::Critical => "CRIT",
        Severity::Neutral => "INFO",
    }
}

fn render_alert_block(frame: &mut Frame, area: Rect, app: &AppState) {
    let t = theme::get();

    let sev = if app.alerts.len() > 1 && app.alerts[0].contains("\u{26a1}") {
        Severity::Critical
    } else if app.alerts.len() > 1 {
        Severity::Warn
    } else if app.alerts[0].contains("stable") {
        Severity::Ok
    } else {
        Severity::Warn
    };

    let health_sev = Severity::from_health(app.health_score as f64);
    let status_text = match health_sev {
        Severity::Ok => "System healthy. No action needed.",
        Severity::Warn => "Needs attention. Watch active thresholds.",
        Severity::Critical => "Critical pressure. Investigate now.",
        Severity::Neutral => "Monitoring system state.",
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("System: ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!(
                    "{} {}% {}  ",
                    health_sev.symbol(),
                    app.health_score,
                    severity_label(health_sev)
                ),
                Style::default()
                    .fg(severity_color(health_sev))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(status_text, Style::default().fg(t.text)),
        ]),
        Line::from(vec![
            Span::styled("Root cause: ", Style::default().fg(t.overlay0)),
            Span::styled(
                app.root_causes
                    .first()
                    .map(|cause| format!("{} {}", cause.severity.symbol(), cause.title))
                    .unwrap_or_else(|| String::from("○ Analysis pending")),
                Style::default()
                    .fg(app
                        .root_causes
                        .first()
                        .map(|cause| severity_color(cause.severity))
                        .unwrap_or(t.overlay1))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.root_causes
                    .first()
                    .map(|cause| format!("  |  {}", cause.detail))
                    .unwrap_or_default(),
                Style::default().fg(t.text),
            ),
        ]),
        Line::from(vec![
            Span::styled("Data quality: ", Style::default().fg(t.overlay0)),
            Span::styled(
                app.sample_status(),
                Style::default()
                    .fg(sample_status_color(app.sample_status()))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  |  Reads ok/fail: {}/{}  |  Updated {:.1}s ago",
                    app.successful_reads,
                    app.failed_reads,
                    app.last_sample_at.elapsed().as_secs_f64()
                ),
                Style::default().fg(t.overlay1),
            ),
        ]),
    ];

    if !app.degraded_sources.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Missing sources: ", Style::default().fg(t.accent_orange)),
            Span::styled(
                app.degraded_sources.join(", "),
                Style::default().fg(t.accent_red),
            ),
        ]));
    }

    for cause in app.root_causes.iter().skip(1).take(2) {
        lines.push(Line::from(vec![
            Span::styled("Cause: ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!("{} {}  ", cause.severity.symbol(), cause.title),
                Style::default()
                    .fg(severity_color(cause.severity))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(cause.detail.clone(), Style::default().fg(t.text)),
        ]));
    }

    if let Some(unit) = app.failed_units.first() {
        lines.push(Line::from(vec![
            Span::styled("Service: ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!("{}  ", unit.unit),
                Style::default()
                    .fg(t.accent_orange)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                truncate(
                    &format!(
                        "{} / {} / {} | {}",
                        unit.load, unit.active, unit.sub, unit.description
                    ),
                    54,
                ),
                Style::default().fg(t.overlay1),
            ),
        ]));
    }

    if let Some(drive) = app
        .storage_health
        .iter()
        .find(|drive| drive.risk != Severity::Ok)
        .or_else(|| app.storage_health.first())
    {
        let temp = drive
            .temp_c
            .map(|value| format!("{value:.0}\u{b0}C"))
            .unwrap_or_else(|| String::from("temp N/A"));
        lines.push(Line::from(vec![
            Span::styled("Storage: ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!("{} {}  ", drive.risk.symbol(), drive.device),
                Style::default()
                    .fg(severity_color(drive.risk))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} | {} | {}",
                    temp,
                    truncate(&drive.model, 18),
                    truncate(&drive.note, 26)
                ),
                Style::default().fg(t.overlay1),
            ),
        ]));
    }

    if let Some(gpu) = app.primary_gpu() {
        let usage = gpu
            .usage_pct
            .map(|value| format!("{value:.0}%"))
            .unwrap_or_else(|| String::from("load N/A"));
        let temp = gpu
            .temp_c
            .map(|value| format!("{value:.0}\u{b0}C"))
            .unwrap_or_else(|| String::from("temp N/A"));
        lines.push(Line::from(vec![
            Span::styled("GPU: ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!("{} {}  ", gpu.kind, gpu.card),
                Style::default()
                    .fg(t.accent_purple)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} | {} | {} | {}",
                    truncate(&gpu.model, 24),
                    usage,
                    temp,
                    truncate(&gpu.sensor_source, 18)
                ),
                Style::default().fg(t.overlay1),
            ),
        ]));
    }

    for event in app.events.iter().rev().take(3) {
        lines.push(Line::from(vec![
            Span::styled("Event: ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!("{} {}  ", event.severity.symbol(), event.title),
                Style::default()
                    .fg(severity_color(event.severity))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(truncate(&event.detail, 46), Style::default().fg(t.overlay1)),
        ]));
    }

    let alerts_to_show = (area.height as usize).saturating_sub(4).min(6);
    for alert in app.alerts.iter().take(alerts_to_show) {
        let is_critical = alert.starts_with("\u{26a1}");
        let is_warn = alert.starts_with("\u{26a0}");
        let color = if is_critical {
            t.accent_red
        } else if is_warn {
            t.accent_orange
        } else {
            t.accent_green
        };
        lines.push(Line::from(vec![
            Span::styled("Note: ", Style::default().fg(t.overlay0)),
            Span::styled(alert.clone(), Style::default().fg(color)),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel_block_severity("Status and guidance", sev)),
        area,
    );
}
