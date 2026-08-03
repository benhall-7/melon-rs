use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, watch};

use crate::audio::Audio;
use crate::input::{
    Binding, BindingOutcome, Bindings, BoundaryIndex, BoundaryInput, FrontendCommand,
    InputAccumulator, InputEvent, KeyCombination, SystemAction,
};
use crate::melon::nds::Nds;
use crate::replay::SavestateContextReplay;
use crate::replay::{Replay, SavestateContext};
use crate::utils::localize_pathbuf;
use crate::EmuStateChange;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReplayState {
    Recording,
    Playing,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Request {
    WriteSavestate(PathBuf),
    ReadSavestate(PathBuf),
    WriteRam(PathBuf),
    WriteReplay,
    WriteSavedata(PathBuf),
}

/// A file the emulator wants written, handed off so a multi-megabyte write never
/// lands inside a frame.
#[derive(Debug, PartialEq, Clone)]
pub struct Save {
    pub path: PathBuf,
    pub contents: Vec<u8>,
}

/// Both screens as the console last drew them, in melonDS's BGRA order.
#[derive(Debug, PartialEq, Clone)]
pub struct Frames {
    pub top: Vec<u8>,
    pub bottom: Vec<u8>,
}

impl Frames {
    /// Bytes in one screen.
    const SIZE: usize = 256 * 192 * 4;

    pub fn blank() -> Self {
        Frames {
            top: vec![0; Self::SIZE],
            bottom: vec![0; Self::SIZE],
        }
    }
}

pub struct Frontend {
    pub nds: Nds,
    replay: Option<(Replay, ReplayState)>,
    /// Finished frames, for whoever is drawing. Sent behind an [`Arc`] so that
    /// publishing one cannot wait on the reader.
    frames: watch::Sender<Arc<Frames>>,
    audio: Audio,
    bindings: Bindings,
    inputs: InputAccumulator,
}

impl Frontend {
    pub fn new(
        cart: Vec<u8>,
        save: Option<Vec<u8>>,
        time: DateTime<Utc>,
        audio: Audio,
        key_map: HashMap<KeyCombination, Binding>,
        replay: Option<(Replay, ReplayState)>,
        frames: watch::Sender<Arc<Frames>>,
    ) -> Self {
        let mut nds = Nds::new();

        nds.set_nds_cart(&cart, save.as_deref());
        nds.set_time(time);

        println!("Needs direct boot? {:?}", nds.needs_direct_boot());

        if nds.needs_direct_boot() {
            nds.setup_direct_boot(String::from("TEMP"));
        }

        nds.start();

        Frontend {
            nds,
            audio,
            bindings: Bindings::new(key_map),
            inputs: InputAccumulator::new(),
            replay,
            frames,
        }
    }

    pub fn handle_input_event(
        &mut self,
        event: InputEvent,
        state_tx: &watch::Sender<Option<EmuStateChange>>,
        request_tx: &mpsc::Sender<Request>,
    ) {
        match self.bindings.handle(event) {
            Some(BindingOutcome::Console(change)) => self.inputs.apply(change),
            Some(BindingOutcome::Command(command)) => {
                self.run_command(command, state_tx, request_tx)
            }
            None => {}
        }
    }

    fn run_command(
        &mut self,
        command: FrontendCommand,
        state_tx: &watch::Sender<Option<EmuStateChange>>,
        request_tx: &mpsc::Sender<Request>,
    ) {
        match command {
            FrontendCommand::PlayPause => {
                state_tx.send(Some(EmuStateChange::PlayPause)).unwrap();
            }
            FrontendCommand::Step => {
                state_tx.send(Some(EmuStateChange::Step)).unwrap();
            }
            FrontendCommand::WriteSavedata(path) => {
                request_tx
                    .try_send(Request::WriteSavedata(path.into()))
                    .unwrap();
            }
            FrontendCommand::ReadSavestate(path) => {
                request_tx
                    .try_send(Request::ReadSavestate(path.into()))
                    .unwrap();
            }
            FrontendCommand::WriteSavestate(path) => {
                request_tx
                    .try_send(Request::WriteSavestate(path.into()))
                    .unwrap();
            }
            FrontendCommand::WriteMainRam(path) => {
                request_tx.try_send(Request::WriteRam(path.into())).unwrap();
            }
            FrontendCommand::ToggleReplayMode => {
                if let Some(state) = self.replay.as_mut() {
                    match state.1 {
                        ReplayState::Playing => {
                            state.1 = ReplayState::Recording;
                            println!("Switched to write mode");
                        }
                        ReplayState::Recording => {
                            state.1 = ReplayState::Playing;
                            println!("Switched to read mode");
                        }
                    }
                }
            }
            FrontendCommand::SaveReplay => {
                request_tx.try_send(Request::WriteReplay).unwrap();
            }
        }
    }

    pub fn run_frame(&mut self) {
        let input = self.select_input();
        self.record(&input);
        self.apply_input(&input);

        self.nds.run_frame();

        self.update_audio();
        self.publish_frames();
    }

    /// Closes the current window and decides what the console will see.
    ///
    /// The accumulator is sampled even during playback, so that host input
    /// accumulated while watching cannot leak into the first recorded window
    /// after switching to recording.
    fn select_input(&mut self) -> BoundaryInput {
        let boundary = self.nds.current_frame() as usize;
        let live = self.inputs.sample(BoundaryIndex(boundary as u64));

        match &self.replay {
            Some((replay, ReplayState::Playing)) if boundary < replay.inputs.len() => {
                replay.inputs[boundary].clone()
            }
            _ => live,
        }
    }

    fn record(&mut self, input: &BoundaryInput) {
        let boundary = self.nds.current_frame() as usize;

        if let Some((replay, ReplayState::Recording)) = self.replay.as_mut() {
            if boundary <= replay.inputs.len() {
                replay.inputs.splice(boundary.., [input.clone()]);
            } else {
                println!(
                    "WARNING: the replay is in recording mode, but \
                                cannot record new inputs, because the current \
                                frame extends beyond the last recorded frame"
                )
            }
        }
    }

    /// Hands one boundary's input to the core, held state first and one-shot
    /// actions second, so an action always observes the state it was sampled
    /// alongside.
    fn apply_input(&mut self, input: &BoundaryInput) {
        self.nds.set_key_mask(input.state.buttons);
        match input.state.touch {
            Some(point) => self.nds.touch_screen(point.x as u16, point.y as u16),
            None => self.nds.release_screen(),
        }
        // Opening the lid raises an IRQ every time melonDS is told to do it, so
        // the write has to be an edge even though the state we hold is absolute.
        if input.state.lid_closed != self.nds.is_lid_closed() {
            self.nds.set_lid_closed(input.state.lid_closed);
        }

        for action in &input.actions {
            match action {
                SystemAction::Reset => {
                    self.nds.reset();
                    self.nds.start();
                }
                // These need melonDS shims that do not exist yet, per item 9.
                // No binding produces them, so this cannot fire today.
                SystemAction::PowerCycle
                | SystemAction::InsertCartridge
                | SystemAction::EjectCartridge => {
                    println!("WARNING: the {action:?} action is not implemented yet")
                }
            }
        }
    }

    pub fn update_audio(&mut self) {
        let skew = self.audio.submit(&self.nds.read_audio_output());

        self.nds.set_audio_output_skew(skew);
    }

    /// Hands the finished screens to whoever is drawing.
    fn publish_frames(&mut self) {
        let mut frames = Frames::blank();

        self.nds.update_framebuffers(&mut frames.top, false);
        self.nds.update_framebuffers(&mut frames.bottom, true);

        // A closed window outliving the last frame is not worth reporting.
        let _ = self.frames.send(Arc::new(frames));
    }

    pub fn read_savestate(&mut self, file: String) {
        let path = localize_pathbuf(file);
        let localized = path.to_string_lossy().into_owned();

        let mut context_path = path.into_os_string();
        context_path.push(".context");
        let context_path = PathBuf::from(context_path);

        let context_str = std::fs::read_to_string(&context_path).ok();
        if context_str.is_none() {
            println!("Couldn't read savestate: {}", context_path.display());
            return;
        }
        let context_result = serde_yaml::from_str(context_str.as_ref().unwrap());
        if context_result.is_err() {
            println!("Couldn't read savestate context: {}", context_str.unwrap());
            return;
        }
        let context: SavestateContext = context_result.unwrap();

        match (&mut self.replay, context.replay) {
            (Some(replay), Some(replay_context)) => {
                if replay_context.name == replay.0.name {
                    replay.0.inputs = replay_context.inputs;
                    assert!(self.nds.read_savestate(localized));
                } else {
                    println!("The savestate couldn't be loaded. The savestate belongs to a different replay")
                }
            }
            (Some(_), None) => println!("The savestate couldn't be loaded. There is a replay running, but the savestate doesn't belong to one"),
            (None, Some(_)) => println!("The savestate couldn't be loaded. There is no replay running, and the savestate belongs to a replay"),
            (None, None) => {
                assert!(self.nds.read_savestate(localized));
            },
        }
    }

    /// Snapshots the console and the replay it belongs to, for writing elsewhere.
    pub fn savestate(&mut self, file: String) -> Vec<Save> {
        let path = localize_pathbuf(file);

        let mut context_path = path.clone().into_os_string();
        context_path.push(".context");

        let context = SavestateContext {
            replay: self.replay.as_ref().map(|replay| SavestateContextReplay {
                name: replay.0.name.clone(),
                inputs: replay.0.inputs.clone(),
            }),
        };

        vec![
            Save {
                path,
                contents: self.nds.savestate(),
            },
            Save {
                path: context_path.into(),
                contents: serde_yaml::to_string(&context).unwrap().into_bytes(),
            },
        ]
    }

    /// The replay as it stands, for writing elsewhere.
    pub fn replay_save(&self) -> Option<Save> {
        let (replay, _) = self.replay.as_ref()?;

        Some(Save {
            path: replay.name.clone(),
            contents: serde_yaml::to_string(replay).unwrap().into_bytes(),
        })
    }
}

