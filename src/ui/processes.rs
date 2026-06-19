use crate::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table, TableState},
    Frame,
};

use super::common::*;
use super::theme;

pub fn processes_tab(frame: &mut Frame, area: Rect, app: &AppState, table_state: &mut TableState) {
    let t = theme::get();

    let show_search = app.is_search_mode || !app.process_search.is_empty();
    let header_height = if show_search { 5 } else { 3 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(header_height), Constraint::Min(6)])
        .split(area);

    let header_chunks = if show_search {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(2)])
            .split(chunks[0])
            .to_vec()
    } else {
        vec![chunks[0]]
    };

    let summary_line = Line::from(vec![
        styled(format!("\u{2630} {} processes", app.process_count), t.text),
        styled("  Sort: ", t.overlay0),
        styled(app.process_sort.label(), t.accent_blue).add_modifier(Modifier::BOLD),
        styled(
            "  [S] cycle  [/] search  [K] kill  [\u{2191}/\u{2193}] select",
            t.overlay1,
        ),
    ]);
    frame.render_widget(
        Paragraph::new(summary_line).block(panel_block("\u{2630} Process Overview")),
        header_chunks[0],
    );

    if show_search {
        let cursor = if app.is_search_mode { "█" } else { "" };
        let search_line = Line::from(vec![
            Span::styled(
                "  🔎 Search: ",
                Style::default()
                    .fg(t.accent_blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}{}", app.process_search, cursor),
                Style::default().fg(t.text),
            ),
            Span::styled("  (Esc to clear/exit)", Style::default().fg(t.overlay0)),
        ]);
        frame.render_widget(Paragraph::new(search_line), header_chunks[1]);
    }

    let sorted = app.filtered_processes();

    let p_rows = sorted
        .iter()
        .map(|p| {
            let is_risk = p.is_high_risk;
            let is_dev = p.is_dev;
            let pid_color = if is_risk { t.accent_red } else { t.text };
            let cpu_color = if is_risk { t.accent_red } else { t.accent_teal };
            let spark = app
                .process_history
                .get(&p.pid)
                .map(|h| {
                    h.iter()
                        .rev()
                        .take(10)
                        .rev()
                        .map(|&v| sparkline_chars(v))
                        .collect::<String>()
                })
                .unwrap_or_default();

            let sev = if is_risk {
                crate::types::Severity::Critical
            } else {
                crate::types::Severity::Ok
            };

            let name_color = if is_risk {
                t.accent_red
            } else if is_dev {
                t.accent_teal
            } else {
                t.text
            };
            let name_prefix = if is_dev { "[Dev] " } else { "" };

            Row::new(vec![
                Cell::from(Span::styled(
                    format!("{:<7}", p.pid),
                    Style::default().fg(pid_color),
                )),
                Cell::from(Span::styled(
                    format!("{:>5.1}% ", p.cpu_pct),
                    Style::default().fg(cpu_color).add_modifier(if is_risk {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                )),
                Cell::from(Span::styled(
                    format!("{:>7.1}M", p.mem_mb),
                    Style::default().fg(t.overlay0),
                )),
                Cell::from(Span::styled(
                    format!("{:>4}", p.threads),
                    Style::default().fg(t.overlay1),
                )),
                Cell::from(Span::styled(
                    truncate(&p.state, 10),
                    Style::default().fg(t.overlay0),
                )),
                Cell::from(Span::styled(
                    format!("{}  {}{}", sev.symbol(), name_prefix, truncate(&p.name, 18)),
                    Style::default().fg(name_color).add_modifier(if is_dev {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                )),
                Cell::from(Span::styled(
                    truncate(&p.reason, 16),
                    Style::default().fg(if p.reason == "Normal" {
                        t.overlay0
                    } else if is_dev {
                        t.accent_teal
                    } else {
                        severity_color(sev)
                    }),
                )),
                Cell::from(Span::styled(spark, Style::default().fg(t.accent_teal))),
            ])
            .style(Style::default().fg(t.text))
        })
        .collect::<Vec<_>>();

    if let Some(pid) = app.confirm_kill_pid {
        let name = app.confirm_kill_name.as_deref().unwrap_or("unknown");
        let confirm_text = Line::from(vec![
            Span::styled(
                format!("  Kill PID {pid} ({name})? "),
                Style::default()
                    .fg(t.accent_red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("[K] confirm  ", Style::default().fg(t.accent_red)),
            Span::styled("[Esc] cancel", Style::default().fg(t.overlay0)),
        ]);
        frame.render_widget(
            Paragraph::new(confirm_text).block(panel_block_severity(
                "\u{26a0} Confirm Kill",
                crate::types::Severity::Critical,
            )),
            chunks[1],
        );
        return;
    }

    if p_rows.is_empty() {
        frame.render_widget(
            Paragraph::new("No process data").block(panel_block("\u{2630} Process List")),
            chunks[1],
        );
    } else {
        *table_state.offset_mut() = table_state.offset().min(p_rows.len().saturating_sub(1));
        let highlight_style = Style::default()
            .fg(Color::Rgb(24, 24, 37))
            .bg(t.accent_blue);

        frame.render_stateful_widget(
            Table::new(
                p_rows,
                [
                    Constraint::Length(8),
                    Constraint::Length(8),
                    Constraint::Length(9),
                    Constraint::Length(5),
                    Constraint::Length(11),
                    Constraint::Min(16),
                    Constraint::Length(17),
                    Constraint::Length(12),
                ],
            )
            .header(Row::new(vec![
                Cell::from(header_col("PID")),
                Cell::from(header_col("CPU%")),
                Cell::from(header_col("MEM")),
                Cell::from(header_col("THR")),
                Cell::from(header_col("State")),
                Cell::from(header_col("Command")),
                Cell::from(header_col("Why")),
                Cell::from(header_col("Spark")),
            ]))
            .block(panel_block("\u{2630} Process List"))
            .column_spacing(1)
            .highlight_style(highlight_style)
            .highlight_symbol("  \u{25b6} "),
            chunks[1],
            table_state,
        );
    }
}

use ratatui::widgets::Cell;
