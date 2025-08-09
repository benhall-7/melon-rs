use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use glium::glutin::event::{ModifiersState, VirtualKeyCode};
use serde::{Deserialize, Serialize};

use crate::args::{Args, Commands};
use crate::frontend::{EmuInput, KeyPressAction, NdsAction, ReplayState};
use crate::replay::{Replay, ReplaySource};

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub default_game_path: Option<PathBuf>,
    pub default_save_path: Option<PathBuf>,
    pub timestamp: Option<DateTime<Utc>>,
    pub key_map: HashMap<EmuInput, KeyPressAction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StartParams {
    pub replay: Option<(Replay, ReplayState)>,
    pub game_name: PathBuf,
    pub save_name: Option<PathBuf>,
    pub start_time: DateTime<Utc>,
}

impl Config {
    pub fn get_start_params(&self, args: Args) -> StartParams {
        let game_name = args.game.as_ref().or(self.default_game_path.as_ref()).map(Clone::clone).expect("No game was selected in the command arguments, and no default game was included in the config");
        // only load a save if the emulation is in "play" mode
        let mut save_name = None;
        let mut replay: Option<(Replay, ReplayState)> = None;

        let mut start_time = self.timestamp.unwrap_or_else(Utc::now);

        match &args.command {
            Commands::Play(play_args) => {
                if !play_args.no_save {
                    save_name = play_args
                        .save
                        .as_ref()
                        .or(self.default_save_path.as_ref())
                        .map(Clone::clone);
                }
            }
            Commands::Replay(replay_args) => {
                replay = Some((
                    serde_yaml::from_str(&std::fs::read_to_string(&replay_args.name).unwrap())
                        .unwrap(),
                    ReplayState::Playing,
                ));
            }
            Commands::Record(record_args) => {
                replay = Some((
                    Replay {
                        name: record_args.name.clone(),
                        author: record_args.author.clone().unwrap_or_default(),
                        source: ReplaySource::SaveFile {
                            path: record_args.save.clone(),
                            timestamp: record_args
                                .timestamp
                                .as_ref()
                                .map(|datetime| {
                                    DateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%.f%z")
                                        .expect("The datetime could not be parsed")
                                })
                                .map(Into::into)
                                .unwrap_or_else(Utc::now),
                        },
                        inputs: vec![],
                    },
                    ReplayState::Recording,
                ))
            }
        }

        if let Some((replay, _)) = &replay {
            match &replay.source {
                ReplaySource::SaveFile { path, timestamp } => {
                    save_name = path.clone();
                    start_time = *timestamp;
                }
            }
        }

        StartParams {
            replay,
            game_name,
            save_name,
            start_time,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_game_path: None,
            default_save_path: None,
            timestamp: None,
            key_map: vec![
                (VirtualKeyCode::W, KeyPressAction::NdsAction(NdsAction::Up)),
                (
                    VirtualKeyCode::A,
                    KeyPressAction::NdsAction(NdsAction::Left),
                ),
                (
                    VirtualKeyCode::S,
                    KeyPressAction::NdsAction(NdsAction::Down),
                ),
                (
                    VirtualKeyCode::D,
                    KeyPressAction::NdsAction(NdsAction::Right),
                ),
                (VirtualKeyCode::I, KeyPressAction::NdsAction(NdsAction::X)),
                (VirtualKeyCode::J, KeyPressAction::NdsAction(NdsAction::Y)),
                (VirtualKeyCode::K, KeyPressAction::NdsAction(NdsAction::B)),
                (VirtualKeyCode::L, KeyPressAction::NdsAction(NdsAction::A)),
                (VirtualKeyCode::Q, KeyPressAction::NdsAction(NdsAction::L)),
                (VirtualKeyCode::P, KeyPressAction::NdsAction(NdsAction::R)),
                (
                    VirtualKeyCode::Space,
                    KeyPressAction::NdsAction(NdsAction::Start),
                ),
                (
                    VirtualKeyCode::X,
                    KeyPressAction::NdsAction(NdsAction::Select),
                ),
                (
                    VirtualKeyCode::X,
                    KeyPressAction::NdsAction(NdsAction::OpenCloseLid),
                ),
                (
                    VirtualKeyCode::Slash,
                    KeyPressAction::NdsAction(NdsAction::OpenCloseLid),
                ),
                (VirtualKeyCode::Comma, KeyPressAction::PlayPlause),
                (VirtualKeyCode::Period, KeyPressAction::Step),
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
            .chain(vec![(
                EmuInput {
                    key_code: VirtualKeyCode::S,
                    modifiers: ModifiersState::CTRL,
                },
                KeyPressAction::Save(String::from("save.bin")),
            )])
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
    action: KeyPressAction,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub default_game_path: Option<PathBuf>,
    pub default_save_path: Option<PathBuf>,
    pub timestamp: Option<DateTime<Utc>>,
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
            default_game_path: value.default_game_path,
            default_save_path: value.default_save_path,
            timestamp: value.timestamp,
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
            default_game_path: value.default_game_path,
            default_save_path: value.default_save_path,
            timestamp: value.timestamp,
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
