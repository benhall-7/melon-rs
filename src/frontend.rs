use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use glium::glutin::event::{ElementState, WindowEvent};
use serde::{Deserialize, Serialize};
use winit::event::{ModifiersState, MouseButton, VirtualKeyCode};

use crate::melon::nds::input::{NdsInput, NdsKey, NdsKeyMask};
use crate::melon::nds::Nds;
use crate::melon::save;
use crate::replay::SavestateContextReplay;
use crate::replay::{Replay, SavestateContext};
use crate::utils::localize_pathbuf;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Run,
    Pause,
    Stop,
    Step,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReplayState {
    Recording,
    Playing,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum KeyPressAction {
    NdsKey(NdsKey),
    PlayPlause,
    Step,
    Save(String),
    ReadSavestate(String),
    WriteSavestate(String),
    ToggleReplayMode,
    SaveReplay,
    WriteMainRAM(String),
}

#[derive(Debug, Default)]
pub struct Requests {
    pub savestate_write_request: Option<String>,
    pub savestate_read_request: Option<String>,
    pub ram_write_request: Option<String>,
    pub replay_save_request: bool,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct EmuInput {
    pub key_code: VirtualKeyCode,
    pub modifiers: ModifiersState,
}

#[derive(Debug)]
pub struct Frontend {
    pub top_frame: [u8; 256 * 192 * 4],
    pub bottom_frame: [u8; 256 * 192 * 4],
    pub audio: Arc<Mutex<Vec<i16>>>,
    pub key_map: HashMap<EmuInput, KeyPressAction>,
    pub key_modifiers: ModifiersState,
    pub nds_input: NdsKeyMask,
    pub cursor: Option<(u8, u8)>,
    pub cursor_pressed: bool,
    pub state: EmuState,
    pub replay: Option<(Replay, ReplayState)>,
    pub requests: Requests,
}

impl Frontend {
    pub fn new(
        audio: Arc<Mutex<Vec<i16>>>,
        key_map: HashMap<EmuInput, KeyPressAction>,
        replay: Option<(Replay, ReplayState)>,
    ) -> Self {
        Frontend {
            top_frame: [0; 256 * 192 * 4],
            bottom_frame: [0; 256 * 192 * 4],
            audio,
            key_map,
            nds_input: NdsKeyMask::empty(),
            key_modifiers: ModifiersState::empty(),
            cursor: None,
            cursor_pressed: false,
            state: EmuState::Pause,
            replay,
            requests: Default::default(),
        }
    }

    // TODO: these may be pretty expensive to run from inside the emu lock
    pub fn read_savestate(&mut self, nds: &mut Nds, file: String) {
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
                    assert!(nds.read_savestate(localized));
                } else {
                    println!("The savestate couldn't be loaded. The savestate belongs to a different replay")
                }
            }
            (Some(_), None) => println!("The savestate couldn't be loaded. There is a replay running, but the savestate doesn't belong to one"),
            (None, Some(_)) => println!("The savestate couldn't be loaded. There is no replay running, and the savestate belongs to a replay"),
            (None, None) => {
                assert!(nds.read_savestate(localized));
            },
        }
    }

    pub fn write_savestate(&mut self, nds: &mut Nds, file: String) {
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

        assert!(nds.write_savestate(localized));
    }

    /// Run a window event through the frontend.
    /// Returns true when the process should end.
    pub fn window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CloseRequested => {
                self.state = EmuState::Stop;
                return true;
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.key_modifiers = modifiers;
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if input.virtual_keycode.is_none() {
                    return false;
                }
                let modifiers = self.key_modifiers;

                let emu_key = match self.key_map.get(&EmuInput {
                    key_code: input.virtual_keycode.unwrap(),
                    modifiers,
                }) {
                    Some(action) => action.to_owned(),
                    None => return false,
                };

                let state = input.state;
                match emu_key {
                    KeyPressAction::NdsKey(nds_key) => match state {
                        ElementState::Pressed => self.nds_input.insert(nds_key.into()),
                        ElementState::Released => self.nds_input.remove(nds_key.into()),
                    },
                    KeyPressAction::PlayPlause => {
                        if let ElementState::Released = state {
                            return false;
                        }
                        let emu_state = &mut self.state;
                        match *emu_state {
                            EmuState::Pause | EmuState::Step => *emu_state = EmuState::Run,
                            EmuState::Run => *emu_state = EmuState::Pause,
                            EmuState::Stop => {}
                        }
                    }
                    KeyPressAction::Step => {
                        if let ElementState::Released = state {
                            return false;
                        }
                        let emu_state = &mut self.state;
                        match *emu_state {
                            EmuState::Stop => {}
                            _ => *emu_state = EmuState::Step,
                        }
                    }
                    KeyPressAction::Save(path) => {
                        if let ElementState::Released = state {
                            return false;
                        }
                        // TODO: track this thread
                        spawn(|| save::update_save(path.into()));
                    }
                    KeyPressAction::ReadSavestate(path) => {
                        if let ElementState::Released = state {
                            return false;
                        }
                        self.requests.savestate_read_request = Some(path);
                    }
                    KeyPressAction::WriteSavestate(path) => {
                        if let ElementState::Released = state {
                            return false;
                        }
                        self.requests.savestate_write_request = Some(path);
                    }
                    KeyPressAction::ToggleReplayMode => {
                        if let ElementState::Released = state {
                            return false;
                        }
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
                        if let ElementState::Released = state {
                            return false;
                        }
                        self.requests.replay_save_request = true;
                    }
                    KeyPressAction::WriteMainRAM(path) => {
                        if let ElementState::Released = state {
                            return false;
                        }
                        self.requests.ram_write_request = Some(path);
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
            } => match button {
                MouseButton::Left => {
                    self.cursor_pressed = state == ElementState::Pressed;
                }
                _ => {}
            },
            WindowEvent::CursorMoved {
                device_id,
                position,
                modifiers,
            } => {
                println!("position: {},{}", position.x, position.y);
            }
            _ => {}
        }

        false
    }
}
