use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    pub game: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Play the emulator normally, with no replays
    Play(PlayArgs),
    /// Play a replay
    Replay(ReplayArgs),
    /// Record a new replay
    Record(RecordArgs),
}

#[derive(Debug, Parser)]
pub struct PlayArgs {
    #[arg(short, long)]
    /// The path of the save file to load, overriding the default in the config
    pub save: Option<PathBuf>,

    #[arg(long)]
    /// Disable loading a save file, even if a default is provided by the config
    pub no_save: bool,
}

#[derive(Debug, Parser)]
pub struct ReplayArgs {
    /// The path that the replay will be loaded from
    pub name: PathBuf,
}

#[derive(Debug, Parser)]
pub struct RecordArgs {
    /// The path that the replay will be saved to
    pub name: PathBuf,

    /// The path of the save file to load. Defaults to no save
    #[arg(long, short)]
    pub save: Option<PathBuf>,

    /// The timestamp to begin emulation at. Defaults to current time.
    /// Datetimes should be written in the format `YYYY-MM-DDT12:34:56+0000`
    #[arg(long, short)]
    pub timestamp: Option<String>,

    /// The author of the recording. Defaults to empty string
    #[arg(long, short)]
    pub author: Option<String>,
}
