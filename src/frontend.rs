use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, watch};
use winit::event::{ModifiersState, VirtualKeyCode};

use crate::melon::nds::input::{NdsInputState, NdsKey, NdsKeyboardInput};
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum KeyPressAction {
    NdsAction(NdsAction),
    PlayPlause,
    Step,
    Save(String),
    ReadSavestate(String),
    WriteSavestate(String),
    ToggleReplayMode,
    SaveReplay,
    WriteMainRAM(String),
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum NdsAction {
    // NDS buttons
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
    R,
    L,
    X,
    Y,
    // other NDS inputs
    OpenCloseLid,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct EmuInput {
    pub key_code: VirtualKeyCode,
    pub modifiers: ModifiersState,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InputEvent {
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode),
    CursorMove(Option<(u8, u8)>),
    MouseDown,
    MouseUp,
    KeyModifierChange(ModifiersState),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Request {
    WriteSavestate(PathBuf),
    ReadSavestate(PathBuf),
    WriteRam(PathBuf),
    WriteReplay,
    WriteSavedata(PathBuf),
}

pub struct Frontend {
    pub nds: Nds,
    pub top_frame: [u8; 256 * 192 * 4],
    pub bottom_frame: [u8; 256 * 192 * 4],
    pub audio: Arc<Mutex<Vec<i16>>>,
    pub key_map: HashMap<EmuInput, KeyPressAction>,
    pub key_modifiers: ModifiersState,
    pub held_actions: HashMap<VirtualKeyCode, KeyPressAction>,
    pub nds_input: NdsInputState,
    pub cursor: Option<(u8, u8)>,
    pub replay: Option<(Replay, ReplayState)>,
}

impl Frontend {
    pub fn new(
        cart: Vec<u8>,
        save: Option<Vec<u8>>,
        time: DateTime<Utc>,
        audio: Arc<Mutex<Vec<i16>>>,
        key_map: HashMap<EmuInput, KeyPressAction>,
        replay: Option<(Replay, ReplayState)>,
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
            top_frame: [0; 256 * 192 * 4],
            bottom_frame: [0; 256 * 192 * 4],
            audio,
            key_map,
            key_modifiers: ModifiersState::empty(),
            held_actions: HashMap::new(),
            nds_input: NdsInputState::new(),
            cursor: None,
            replay,
        }
    }

    pub fn handle_input_event(
        &mut self,
        event: InputEvent,
        state_rx: &watch::Sender<Option<EmuStateChange>>,
        request_rx: &mpsc::Sender<Request>,
    ) {
        match event {
            InputEvent::KeyDown(key_code) => {
                let modifiers = self.key_modifiers;

                let emu_key = match self.key_map.get(&EmuInput {
                    key_code,
                    modifiers,
                }) {
                    Some(action) => action.to_owned(),
                    None => return,
                };

                self.held_actions.insert(key_code, emu_key.clone());

                match emu_key {
                    KeyPressAction::NdsAction(nds_action) => {
                        NdsKeyboardInput::from(nds_action)
                            .press()
                            .map(|input| self.nds_input.register_input(input));
                    }
                    KeyPressAction::PlayPlause => {
                        state_rx.send(Some(EmuStateChange::PlayPause)).unwrap();
                    }
                    KeyPressAction::Step => {
                        state_rx.send(Some(EmuStateChange::Step)).unwrap();
                    }
                    // TODO: implement these actions
                    KeyPressAction::Save(path) => {
                        request_rx
                            .try_send(Request::WriteSavedata(path.into()))
                            .unwrap();
                    }
                    KeyPressAction::ReadSavestate(path) => {
                        request_rx
                            .try_send(Request::ReadSavestate(path.into()))
                            .unwrap();
                    }
                    KeyPressAction::WriteSavestate(path) => {
                        request_rx
                            .try_send(Request::WriteSavestate(path.into()))
                            .unwrap();
                    }
                    KeyPressAction::ToggleReplayMode => {
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
                    KeyPressAction::SaveReplay => {
                        request_rx.try_send(Request::WriteReplay).unwrap();
                    }
                    KeyPressAction::WriteMainRAM(path) => {
                        request_rx.try_send(Request::WriteRam(path.into())).unwrap();
                    }
                }
            }
            InputEvent::KeyUp(key_code) => {
                let emu_key = match self.held_actions.remove(&key_code) {
                    Some(action) => action.to_owned(),
                    None => return,
                };

                match emu_key {
                    KeyPressAction::NdsAction(nds_action) => {
                        NdsKeyboardInput::from(nds_action)
                            .release()
                            .map(|input| self.nds_input.register_input(input));
                    }
                    _ => {}
                }
            }
            InputEvent::CursorMove(coord) => self.cursor = coord,
            InputEvent::MouseDown => self.nds_input.touch = self.cursor,
            InputEvent::MouseUp => self.nds_input.touch = None,
            InputEvent::KeyModifierChange(mods) => self.key_modifiers = mods,
        }
    }

    pub fn run_frame(&mut self) {
        let inputs = self.get_nds_inputs();
        self.record_replay_nds_input();
        self.set_inputs(inputs);

        self.nds.run_frame();

        // something feels off about having this here...
        self.nds_input.lid_changed = false;

        self.update_audio();
        self.update_framebuffers();
    }

    pub fn get_nds_inputs(&self) -> NdsInputState {
        let emu_inputs = self.nds_input;
        let current_frame = self.nds.current_frame() as usize;
        if let Some((replay, ReplayState::Playing)) = &self.replay {
            if current_frame < replay.inputs.len() {
                return replay.inputs[current_frame];
            }
        }
        return emu_inputs;
    }

    pub fn record_replay_nds_input(&mut self) {
        let emu_inputs = self.nds_input;
        let current_frame = self.nds.current_frame() as usize;

        if let Some((replay, ReplayState::Recording)) = self.replay.as_mut() {
            if current_frame <= replay.inputs.len() {
                replay.inputs.splice(current_frame.., [emu_inputs]);
            } else {
                println!(
                    "WARNING: the replay is in recording mode, but \
                                cannot record new inputs, because the current \
                                frame extends beyond the last recorded frame"
                )
            }
        }
    }

    pub fn set_inputs(&mut self, inputs: NdsInputState) {
        // set key press bitmask
        self.nds.set_key_mask(inputs.keys);
        // set touch screen
        match inputs.touch {
            Some((x, y)) => self.nds.touch_screen(x as u16, y as u16),
            None => self.nds.release_screen(),
        }
        // optionally open or close lid
        if inputs.lid_changed {
            let lid_open = !self.nds.is_lid_closed();
            self.nds.set_lid_closed(lid_open);
        }
    }

    pub fn update_audio(&mut self) {
        let audio_out = self.nds.read_audio_output();
        self.audio
            .lock()
            .map(|mut stream| stream.extend(audio_out))
            .expect("failed to access audio lock");
    }

    pub fn update_framebuffers(&mut self) {
        let mut top = self.top_frame;
        let mut bottom = self.bottom_frame;
        self.nds.update_framebuffers(&mut top, false);
        self.nds.update_framebuffers(&mut bottom, true);
        self.top_frame = top;
        self.bottom_frame = bottom;
    }

    pub fn read_savestate(&mut self, file: String) {
        let localized = localize_pathbuf(file).to_string_lossy().into_owned();

        let mut raw: OsString = localized.clone().into();
        raw.push(".context");
        let context_path = PathBuf::from(raw).to_string_lossy().into_owned();

        let context_str = std::fs::read_to_string(&context_path).ok();
        if context_str.is_none() {
            println!("Couldn't read savestate: {}", context_path);
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

    pub fn write_savestate(&mut self, file: String) {
        let localized = localize_pathbuf(file).to_string_lossy().into_owned();

        let mut raw: OsString = localized.clone().into();
        raw.push(".context");
        let context_path = PathBuf::from(raw).to_string_lossy().into_owned();

        let context = SavestateContext {
            replay: self.replay.as_ref().map(|replay| SavestateContextReplay {
                name: replay.0.name.clone(),
                inputs: replay.0.inputs.clone(),
            }),
        };

        let context_str = serde_yaml::to_string(&context).unwrap();
        std::fs::write(context_path, context_str)
            .expect("Couldn't write savestate context object to file");

        assert!(self.nds.write_savestate(localized));
    }
}

impl From<NdsAction> for NdsKeyboardInput {
    fn from(value: NdsAction) -> Self {
        match value {
            NdsAction::A => NdsKeyboardInput::Key(NdsKey::A),
            NdsAction::B => NdsKeyboardInput::Key(NdsKey::B),
            NdsAction::Select => NdsKeyboardInput::Key(NdsKey::Select),
            NdsAction::Start => NdsKeyboardInput::Key(NdsKey::Start),
            NdsAction::Right => NdsKeyboardInput::Key(NdsKey::Right),
            NdsAction::Left => NdsKeyboardInput::Key(NdsKey::Left),
            NdsAction::Up => NdsKeyboardInput::Key(NdsKey::Up),
            NdsAction::Down => NdsKeyboardInput::Key(NdsKey::Down),
            NdsAction::R => NdsKeyboardInput::Key(NdsKey::R),
            NdsAction::L => NdsKeyboardInput::Key(NdsKey::L),
            NdsAction::X => NdsKeyboardInput::Key(NdsKey::X),
            NdsAction::Y => NdsKeyboardInput::Key(NdsKey::Y),
            NdsAction::OpenCloseLid => NdsKeyboardInput::OpenCloseLid,
        }
    }
}
