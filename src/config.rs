use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use glium::glutin::event::{ModifiersState, VirtualKeyCode};
use serde::{Deserialize, Serialize};

use crate::{
    args::{Args, Commands},
    frontend::{EmuAction, EmuInput, ReplayState},
    melon::nds::input::NdsKey,
    replay::{Replay, ReplaySource},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub default_game_path: Option<PathBuf>,
    pub default_save_path: Option<PathBuf>,
    pub timestamp: Option<DateTime<Utc>>,
    pub key_map: HashMap<EmuInput, EmuAction>,
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
            .chain(vec![(
                EmuInput {
                    key_code: VirtualKeyCode::S,
                    modifiers: ModifiersState::CTRL,
                },
                EmuAction::Save(String::from("save.bin")),
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
    action: EmuAction,
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
