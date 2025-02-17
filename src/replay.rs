use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub name: PathBuf,
    pub author: String,
    pub source: ReplaySource,
    // inputs are determined by a 32-bit bitfield and we can
    // pull it straight from melonDS
    // TODO: this only makes sense for button presses, not screen clicks, lid closing, etc
    pub inputs: Vec<u32>,
}

/// Replays could realistically be played back in 3 ways:
/// from the emulator startup using a consistent save file;
/// from a savestate at any particular frame;
/// or from the emulator startup with no backing state.
/// Using a save file is preferred. Starting a replay from a savestate
/// makes it not possible to prove if game memory was tampered with,
/// while having no consistent source is likely to cause desyncs.
/// 
/// TODO: implement more than just save file recordings
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ReplaySource {
    SaveFile {
        path: Option<PathBuf>,
        timestamp: DateTime<Utc>,
    },
    // Savestate {
    //     path: PathBuf,
    //     start_frame: u32,
    // },
    // None {
    //     timestamp: DateTime<Utc>,
    // },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SavestateContext {
    pub replay: Option<SavestateContextReplay>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SavestateContextReplay {
    pub name: PathBuf,
    pub inputs: Vec<u32>,
}
