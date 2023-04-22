use std::collections::HashMap;

use glium::glutin::event::{ModifiersState, VirtualKeyCode};
use serde::{Deserialize, Serialize};

use crate::melon::nds::input::NdsKey;

pub const EXTERNAL_BIOSENABLE: bool = false;
pub const DLDIENABLE: bool = false;
pub const DLDIREAD_ONLY: bool = false;
pub const DLDIFOLDER_SYNC: bool = false;
pub const DSI_SDENABLE: bool = false;
pub const DSI_SDREAD_ONLY: bool = false;
pub const DSI_SDFOLDER_SYNC: bool = false;
pub const FIRMWARE_OVERRIDE_SETTINGS: bool = false;

pub const DLDISIZE: i32 = 0;
pub const DSI_SDSIZE: i32 = 0;
pub const FIRMWARE_LANGUAGE: i32 = 1;
pub const FIRMWARE_BIRTHDAY_MONTH: i32 = 1;
pub const FIRMWARE_BIRTHDAY_DAY: i32 = 1;
pub const FIRMWARE_FAVOURITE_COLOUR: i32 = 0;
pub const AUDIO_BITRATE: i32 = 0;

pub const BIOS9_PATH: &str = "";
pub const BIOS7_PATH: &str = "";
pub const FIRMWARE_PATH: &str = "";
pub const DSI_BIOS9_PATH: &str = "";
pub const DSI_BIOS7_PATH: &str = "";
pub const DSI_FIRMWARE_PATH: &str = "";
pub const DSI_NANDPATH: &str = "";
pub const DLDISDPATH: &str = "dldi.bin";
pub const DLDIFOLDER_PATH: &str = "";
pub const DSI_SDPATH: &str = "dsisd.bin";
pub const DSI_SDFOLDER_PATH: &str = "";
pub const FIRMWARE_USERNAME: &str = "melon-rs";
pub const FIRMWARE_MESSAGE: &str = "";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum EmuAction {
    NdsKey(NdsKey),
    PlayPlause,
    Step,
    Save(String),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct EmuInput {
    pub key_code: VirtualKeyCode,
    pub modifiers: ModifiersState,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub key_map: HashMap<EmuInput, EmuAction>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            key_map: vec![
                (VirtualKeyCode::K, EmuAction::NdsKey(NdsKey::A)),
                (VirtualKeyCode::M, EmuAction::NdsKey(NdsKey::B)),
                (VirtualKeyCode::J, EmuAction::NdsKey(NdsKey::X)),
                (VirtualKeyCode::N, EmuAction::NdsKey(NdsKey::Y)),
                (VirtualKeyCode::W, EmuAction::NdsKey(NdsKey::Up)),
                (VirtualKeyCode::A, EmuAction::NdsKey(NdsKey::Left)),
                (VirtualKeyCode::S, EmuAction::NdsKey(NdsKey::Down)),
                (VirtualKeyCode::D, EmuAction::NdsKey(NdsKey::Right)),
                (VirtualKeyCode::Q, EmuAction::NdsKey(NdsKey::L)),
                (VirtualKeyCode::P, EmuAction::NdsKey(NdsKey::R)),
                (VirtualKeyCode::Space, EmuAction::NdsKey(NdsKey::Start)),
                (VirtualKeyCode::X, EmuAction::NdsKey(NdsKey::Select)),
                (VirtualKeyCode::Comma, EmuAction::PlayPlause),
                (VirtualKeyCode::Period, EmuAction::Step),
            ]
            .into_iter()
            .map(|basic| {
                (
                    EmuInput {
                        key_code: basic.0,
                        modifiers: ModifiersState::empty(),
                    },
                    basic.1,
                )
            })
            .chain(
                vec![(
                    EmuInput {
                        key_code: VirtualKeyCode::S,
                        modifiers: ModifiersState::CTRL,
                    },
                    EmuAction::Save(String::from("save.bin")),
                )]
                .into_iter(),
            )
            .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EmuInputEntry {
    pub key_code: VirtualKeyCode,
    pub modifiers: Option<ModifiersState>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigKeyMapEntry {
    input: EmuInputEntry,
    action: EmuAction,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub key_map: Vec<ConfigKeyMapEntry>,
}

impl From<EmuInputEntry> for EmuInput {
    fn from(value: EmuInputEntry) -> Self {
        EmuInput {
            key_code: value.key_code,
            modifiers: value.modifiers.unwrap_or_default(),
        }
    }
}

impl From<ConfigFile> for Config {
    fn from(value: ConfigFile) -> Self {
        Config {
            key_map: value
                .key_map
                .into_iter()
                .map(|entry| (entry.input.into(), entry.action))
                .collect(),
        }
    }
}

impl From<Config> for ConfigFile {
    fn from(value: Config) -> Self {
        ConfigFile {
            key_map: value
                .key_map
                .into_iter()
                .map(|(input, action)| ConfigKeyMapEntry {
                    input: input.into(),
                    action,
                })
                .collect(),
        }
    }
}

impl From<EmuInput> for EmuInputEntry {
    fn from(value: EmuInput) -> Self {
        EmuInputEntry {
            key_code: value.key_code,
            modifiers: if value.modifiers.eq(&Default::default()) {
                None
            } else {
                Some(value.modifiers)
            },
        }
    }
}
