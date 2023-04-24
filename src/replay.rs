use std::path::PathBuf;

use serde::{Serialize, Deserialize};

/// Replays could realistically be played back in 3 ways:
/// from the emulator startup using a consistent save file;
/// from a savestate at any particular frame;
/// or from the emulator startup with no backing state.
/// Using a save file is preferred. Starting a replay from a savestate
/// makes it not possible to prove if game memory was tampered with,
/// while having no consistent source is likely to cause desyncs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ReplaySource {
    SaveFile { path: PathBuf },
    Savestate { path: PathBuf, start_frame: u32 },
    None,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub name: PathBuf,
    pub author: String,
    pub source: ReplaySource,
    // inputs are determined by a 32-bit bitfield and we can
    // pull it straight from melonDS
    pub inputs: Vec<u32>,
}
