use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::common;
use super::theme;

pub fn header(frame: &mut Frame, area: Rect, app: &crate::state::AppState) {
    let t = theme::get();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    let title = Span::styled(
        format!("LinWatch v{}", env!("CARGO_PKG_VERSION")),
        Style::default()
            .fg(t.accent_blue)
            .add_modifier(Modifier::BOLD),
    );
    let mut meta_spans = vec![Span::styled(
        if app.terminal_width < 110 {
            format!(
                "Host: {}  |  {} {}  |  SELinux: {}",
                app.system.hostname,
                app.system.os_name,
                app.system.os_version,
                app.system.selinux_mode
            )
        } else {
            format!(
                "System health dashboard  |  Host: {}  |  {} {}  |  SELinux: {}",
                app.system.hostname,
                app.system.os_name,
                app.system.os_version,
                app.system.selinux_mode
            )
        },
        Style::default().fg(t.subtext1),
    )];

    if app.git_modified_files > 0 {
        meta_spans.push(Span::styled("  |  Git: ", Style::default().fg(t.subtext1)));
        meta_spans.push(Span::styled(
            format!("{} modified", app.git_modified_files),
            Style::default()
                .fg(t.accent_yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if app.terminal_width >= 110 {
        meta_spans.push(Span::styled(
            format!(
                "  |  Kernel: {}  |  CPU Sec: {}",
                common::truncate(&app.system.kernel, 14),
                app.system.cpu_vulnerabilities
            ),
            Style::default().fg(t.subtext1),
        ));
    }

    let meta = Line::from(meta_spans);

    let health_sev = crate::types::Severity::from_health(app.health_score as f64);
    let health_color = common::severity_color(health_sev);
    let sample_color = common::sample_status_color(app.sample_status());
    let temp_value = app
        .temp_c
        .map(|temp| format!("{temp:.0}\u{b0}C"))
        .unwrap_or_else(|| String::from("N/A"));
    let temp_state = match app.temp_c {
        Some(temp) if temp >= 80.0 => "Hot",
        Some(temp) if temp >= 70.0 => "Warm",
        Some(_) => "Normal",
        None => "No sensor",
    };
    let gpu_status = app
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
            format!("GPU {} {usage} {temp}", gpu.kind)
        })
        .unwrap_or_else(|| String::from("GPU N/A"));
    let gpu_compact = app
        .primary_gpu()
        .map(|gpu| format!("GPU {}", gpu.kind))
        .unwrap_or_else(|| String::from("GPU N/A"));
    let status_label = match health_sev {
        crate::types::Severity::Ok => "Healthy",
        crate::types::Severity::Warn => "Attention",
        crate::types::Severity::Critical => "Critical",
        crate::types::Severity::Neutral => "Monitoring",
    };

    let right = if app.terminal_width < 100 {
        Line::from(vec![
            Span::styled(
                format!(
                    "{} {status_label} {}%  ",
                    health_sev.symbol(),
                    app.health_score
                ),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Data ", Style::default().fg(t.overlay0)),
            Span::styled(
                app.sample_status(),
                Style::default()
                    .fg(sample_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("Status ", Style::default().fg(t.overlay0)),
            Span::styled(
                format!(
                    "{} {status_label} {}%  ",
                    health_sev.symbol(),
                    app.health_score
                ),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Data ", Style::default().fg(t.overlay0)),
            Span::styled(
                app.sample_status(),
                Style::default()
                    .fg(sample_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Updated ", Style::default().fg(t.overlay1)),
            Span::styled(
                format!("{:.1}s", app.last_sample_at.elapsed().as_secs_f64()),
                Style::default().fg(t.overlay0),
            ),
        ])
    };

    let header_block = common::panel_block("").borders(ratatui::widgets::Borders::NONE);
    let content = vec![Line::from(title), meta];
    frame.render_widget(
        Paragraph::new(content).block(header_block.clone()),
        layout[0],
    );

    let right_block = common::panel_block("").borders(ratatui::widgets::Borders::NONE);
    let mut right_lines = vec![
        right,
        Line::from(Span::styled(
            format!(
                "CPU: {} cores  |  Processes: {}  |  Uptime: {}",
                app.system.cpu_count, app.process_count, app.uptime
            ),
            Style::default().fg(t.overlay0),
        )),
    ];
    if app.terminal_width < 100 {
        right_lines.insert(
            0,
            Line::from(Span::styled(
                format!("CPU {temp_value} | {gpu_compact}"),
                Style::default().fg(t.overlay1),
            )),
        );
    } else if let Some(bat) = app.battery_pct {
        right_lines.insert(
            0,
            Line::from(Span::styled(
                format!(
                    "Battery: {}% {}  |  CPU Temp: {} {}  |  {}  |  Load: {} {} {}",
                    bat,
                    app.battery_status,
                    temp_value,
                    temp_state,
                    gpu_status,
                    app.load_avg[0],
                    app.load_avg[1],
                    app.load_avg[2]
                ),
                Style::default().fg(t.overlay1),
            )),
        );
    } else {
        right_lines.insert(
            0,
            Line::from(Span::styled(
                format!(
                    "CPU Temp: {} {}  |  {}  |  Load: {} {} {}",
                    temp_value,
                    temp_state,
                    gpu_status,
                    app.load_avg[0],
                    app.load_avg[1],
                    app.load_avg[2]
                ),
                Style::default().fg(t.overlay1),
            )),
        );
    }
    frame.render_widget(
        Paragraph::new(right_lines)
            .block(right_block)
            .alignment(Alignment::Right),
        layout[1],
    );
}
