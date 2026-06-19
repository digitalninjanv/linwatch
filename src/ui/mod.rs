mod common;
mod cpu;
mod footer;
mod gpu;
mod header;
mod help;
mod memory;
mod network;
mod overview;
mod processes;
mod storage;
pub mod theme;

use crate::state::AppState;
use crate::types::{AppConfig, ViewTab};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Tabs},
    Frame,
};
use std::time::Instant;

pub fn run_app<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    config: AppConfig,
) -> Result<(), std::io::Error> {
    let mut app = AppState::new(config);
    let mut last_tick = Instant::now();
    let mut was_resizing = false;
    let mut process_table_state = ratatui::widgets::TableState::default();

    loop {
        terminal.draw(|frame| {
            app.terminal_width = frame.size().width;
            ui(frame, &app, &mut process_table_state);
        })?;

        let timeout = app
            .refresh_rate()
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    was_resizing = false;
                    if app.is_search_mode {
                        match key.code {
                            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                                app.is_search_mode = false;
                            }
                            crossterm::event::KeyCode::Backspace => {
                                app.process_search.pop();
                                app.process_selected = 0;
                                process_table_state.select(Some(0));
                            }
                            crossterm::event::KeyCode::Char(c) => {
                                app.process_search.push(c);
                                app.process_selected = 0;
                                process_table_state.select(Some(0));
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            crossterm::event::KeyCode::Esc => {
                                if app.confirm_kill_pid.is_some() {
                                    app.cancel_kill();
                                } else {
                                    break;
                                }
                            }
                            crossterm::event::KeyCode::Char('q') => {
                                break;
                            }
                            crossterm::event::KeyCode::Char('r') => app.update(),
                            crossterm::event::KeyCode::Char('h') => app.show_help = !app.show_help,
                            crossterm::event::KeyCode::Char('+')
                            | crossterm::event::KeyCode::Char('=') => app.faster_refresh(),
                            crossterm::event::KeyCode::Char('-')
                            | crossterm::event::KeyCode::Char('_') => app.slower_refresh(),
                            crossterm::event::KeyCode::Tab => {
                                app.active_tab = app.active_tab.next()
                            }
                            crossterm::event::KeyCode::BackTab => {
                                app.active_tab = app.active_tab.prev();
                            }
                            crossterm::event::KeyCode::Char('1') => {
                                app.active_tab = ViewTab::Overview;
                            }
                            crossterm::event::KeyCode::Char('2') => {
                                app.active_tab = ViewTab::Cpu;
                            }
                            crossterm::event::KeyCode::Char('3') => {
                                app.active_tab = ViewTab::Gpu;
                            }
                            crossterm::event::KeyCode::Char('4') => {
                                app.active_tab = ViewTab::Memory;
                            }
                            crossterm::event::KeyCode::Char('5') => {
                                app.active_tab = ViewTab::Storage;
                            }
                            crossterm::event::KeyCode::Char('6') => {
                                app.active_tab = ViewTab::Network;
                            }
                            crossterm::event::KeyCode::Char('7') => {
                                app.active_tab = ViewTab::Processes;
                            }
                            crossterm::event::KeyCode::Char('s') => app.process_sort.cycle(),
                            crossterm::event::KeyCode::Char('/') => {
                                if app.active_tab == ViewTab::Processes {
                                    app.is_search_mode = true;
                                    app.process_search.clear();
                                    app.process_selected = 0;
                                    process_table_state.select(Some(0));
                                }
                            }
                            crossterm::event::KeyCode::Char('k') => {
                                if app.active_tab == ViewTab::Processes {
                                    app.request_kill();
                                    process_table_state.select(Some(app.process_selected));
                                }
                            }
                            crossterm::event::KeyCode::Up => {
                                app.process_select_prev();
                                process_table_state.select(Some(app.process_selected));
                            }
                            crossterm::event::KeyCode::Down => {
                                app.process_select_next();
                                process_table_state.select(Some(app.process_selected));
                            }
                            _ => {}
                        }
                    }
                }
                crossterm::event::Event::Resize(_, _) => was_resizing = true,
                _ => {}
            }
        }

        if last_tick.elapsed() >= app.refresh_rate() && !was_resizing {
            app.update();
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn ui(frame: &mut Frame, app: &AppState, process_table_state: &mut ratatui::widgets::TableState) {
    let t = theme::get();

    let root = frame.size();
    let shell = Block::default().style(Style::default().bg(t.bg_dark));
    frame.render_widget(shell, root);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(root);

    header::header(frame, chunks[0], app);
    render_tabs(frame, chunks[1], app);
    render_tab_content(frame, chunks[2], app, process_table_state);
    footer::footer(frame, chunks[3], app);

    if app.show_help {
        help::help(frame, root);
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &AppState) {
    let t = theme::get();
    let titles: Vec<Span> = ViewTab::all()
        .iter()
        .map(|tab| {
            let label = format!(" {} {} ", tab.icon(), tab.label());
            Span::raw(label)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_tab.index())
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(24, 24, 37))
                .bg(t.accent_blue)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(t.overlay0))
        .divider(Span::raw(" "));

    frame.render_widget(tabs, area);
}

fn render_tab_content(
    frame: &mut Frame,
    area: Rect,
    app: &AppState,
    process_table_state: &mut ratatui::widgets::TableState,
) {
    match app.active_tab {
        ViewTab::Overview => overview::overview(frame, area, app),
        ViewTab::Cpu => cpu::cpu_tab(frame, area, app),
        ViewTab::Gpu => gpu::gpu_tab(frame, area, app),
        ViewTab::Memory => memory::memory_tab(frame, area, app),
        ViewTab::Storage => storage::storage_tab(frame, area, app),
        ViewTab::Network => network::network_tab(frame, area, app),
        ViewTab::Processes => {
            processes::processes_tab(frame, area, app, process_table_state);
        }
    }
}
