mod args;
mod file_mode;
mod mic_mode;
mod note_log;

use clap::Parser;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_target(true)
        .init();

    let cli = args::Cli::parse();

    if !cli.requires_one_mode() {
        eprintln!("error: specify --file <WAV_PATH> or --duration <SECONDS>");
        eprintln!("For help: ferrotone-cli --help");
        std::process::exit(1);
    }

    let settings = if let Some(config_path) = &cli.config {
        ferrotone_core::config::Settings::load_from(config_path)
    } else {
        ferrotone_core::config::Settings::load()
    }
    .unwrap_or_else(|e| {
        eprintln!("warning: failed to load settings: {e}, using defaults");
        ferrotone_core::config::Settings::default()
    });

    let result = if let Some(file_path) = &cli.file {
        file_mode::run(file_path, cli.note_log.as_deref(), cli.pitch_log.as_deref())
    } else if let Some(duration) = cli.duration {
        mic_mode::run(
            duration,
            &settings,
            cli.note_log.as_deref(),
            cli.pitch_log.as_deref(),
        )
    } else {
        unreachable!()
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
