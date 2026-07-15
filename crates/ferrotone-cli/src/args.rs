use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "ferrotone-cli",
    about = "FerroTone vocal pitch trainer — CLI mode"
)]
pub struct Cli {
    /// Path to custom config TOML file
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    /// Write note event log (JSON Lines) to this file
    #[arg(short = 'l', long)]
    pub note_log: Option<PathBuf>,

    /// Write per-frame pitch data (JSON Lines) to this file
    #[arg(short = 'p', long)]
    pub pitch_log: Option<PathBuf>,

    /// Input WAV file to process offline (mutually exclusive with --duration)
    #[arg(short = 'f', long, conflicts_with = "duration")]
    pub file: Option<PathBuf>,

    /// Run microphone capture for SECONDS then exit (mutually exclusive with --file)
    #[arg(short = 'd', long, conflicts_with = "file")]
    pub duration: Option<f64>,
}

impl Cli {
    pub fn requires_one_mode(&self) -> bool {
        self.file.is_some() || self.duration.is_some()
    }
}
