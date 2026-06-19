mod collector;
mod state;
mod types;
mod ui;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use std::{
    env, fs, io,
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
};
use types::{AppConfig, MonitorConfig};

fn main() -> Result<(), io::Error> {
    let config = load_config();
    let Some(config) = parse_args(config)? else {
        return Ok(());
    };

    if config.json_once {
        return json_output(&config);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = catch_unwind(AssertUnwindSafe(|| ui::run_app(&mut terminal, config)));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match result {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn json_output(config: &AppConfig) -> Result<(), io::Error> {
    use state::AppState;

    let app = AppState::new_raw(config);
    let snapshot = app.to_snapshot();

    if config.json_pretty {
        serde_json::to_writer_pretty(io::stdout().lock(), &snapshot)
    } else {
        serde_json::to_writer(io::stdout().lock(), &snapshot)
    }
    .map_err(io::Error::other)
}

fn load_config() -> MonitorConfig {
    let config_dir = dirs_config_path();
    let config_path = config_dir.join("fedora-monitor").join("config.toml");

    if !config_path.exists() {
        return MonitorConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Warning: config parse error: {e}");
            MonitorConfig::default()
        }),
        Err(_) => MonitorConfig::default(),
    }
}

fn dirs_config_path() -> std::path::PathBuf {
    if let Ok(home) = env::var("HOME") {
        Path::new(&home).join(".config")
    } else {
        Path::new("/etc").to_path_buf()
    }
}

fn parse_args(config: MonitorConfig) -> Result<Option<AppConfig>, io::Error> {
    let mut app_config = AppConfig {
        refresh_index: resolve_interval_index(config.refresh_interval.as_deref()),
        config,
        json_once: false,
        json_pretty: false,
    };
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_cli_help();
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("fedora-monitor {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "-i" | "--interval" => {
                let Some(value) = args.next() else {
                    return Err(invalid_arg("--interval requires a value"));
                };
                app_config.refresh_index = parse_interval_index(&value)?;
            }
            _ if arg.starts_with("--interval=") => {
                let value = arg.trim_start_matches("--interval=");
                app_config.refresh_index = parse_interval_index(value)?;
            }
            "--json" => {
                app_config.json_once = true;
            }
            "--json-pretty" => {
                app_config.json_once = true;
                app_config.json_pretty = true;
            }
            _ => return Err(invalid_arg(format!("unknown argument: {arg}"))),
        }
    }

    Ok(Some(app_config))
}

fn resolve_interval_index(value: Option<&str>) -> usize {
    match value {
        Some(v) => parse_interval_index(v).unwrap_or(1),
        None => 1,
    }
}

fn parse_interval_index(value: &str) -> Result<usize, io::Error> {
    let normalized = value.trim().to_ascii_lowercase();
    let millis = if let Some(value) = normalized.strip_suffix("ms") {
        value.trim().parse::<u64>().ok()
    } else if let Some(value) = normalized.strip_suffix('s') {
        value
            .trim()
            .parse::<f64>()
            .ok()
            .map(|seconds| (seconds * 1_000.0).round() as u64)
    } else {
        normalized.parse::<u64>().ok()
    };

    let Some(millis) = millis else {
        return Err(invalid_arg(format!("invalid interval value: {value}")));
    };

    types::REFRESH_INTERVALS_MS
        .iter()
        .position(|supported| *supported == millis)
        .ok_or_else(|| {
            invalid_arg(format!(
                "unsupported interval: {value}. Supported: 500ms, 750ms, 1s, 2s, 5s"
            ))
        })
}

fn invalid_arg<T: Into<String>>(message: T) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn print_cli_help() {
    println!(
        "fedora-monitor {}\n\n\
         Usage:\n  fedora-monitor [OPTIONS]\n\n\
         Options:\n  -i, --interval <VALUE>  Refresh interval: 500ms, 750ms, 1s, 2s, 5s\n\
         --json                  Single-shot JSON snapshot to stdout\n\
         --json-pretty           Single-shot pretty-printed JSON\n\
         -h, --help              Show this help\n  -V, --version           Show version\n\n\
         Config:\n  ~/.config/fedora-monitor/config.toml\n\n\
         Keys:\n  Q/Esc  Exit\n  R      Refresh now\n  H      Toggle help\n\
         1-6    Switch tab\n  Tab    Next tab\n  S      Cycle process sort\n\
         K      Kill selected process (press again to confirm)\n\
         /      Search process\n  +/-    Change interval",
        env!("CARGO_PKG_VERSION")
    );
}
