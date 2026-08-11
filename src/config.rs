use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use egui::Key;
use serde::{Deserialize, Serialize};

use crate::frontend::ReplayState;
use crate::input::{
    Binding, ConsoleBinding, ConsoleButton, FrontendCommand, KeyCombination, Modifiers,
};
use crate::replay::Replay;

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub default_game_path: Option<PathBuf>,
    pub default_save_path: Option<PathBuf>,
    pub timestamp: Option<DateTime<Utc>>,
    pub key_map: HashMap<KeyCombination, Binding>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StartParams {
    pub replay: Option<(Replay, ReplayState)>,
    pub game_name: PathBuf,
    pub save_name: Option<PathBuf>,
    pub start_time: DateTime<Utc>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_game_path: None,
            default_save_path: None,
            timestamp: None,
            key_map: vec![
                (Key::W, button(ConsoleButton::Up)),
                (Key::A, button(ConsoleButton::Left)),
                (Key::S, button(ConsoleButton::Down)),
                (Key::D, button(ConsoleButton::Right)),
                (Key::I, button(ConsoleButton::X)),
                (Key::J, button(ConsoleButton::Y)),
                (Key::K, button(ConsoleButton::B)),
                (Key::L, button(ConsoleButton::A)),
                (Key::Q, button(ConsoleButton::L)),
                (Key::P, button(ConsoleButton::R)),
                (Key::Space, button(ConsoleButton::Start)),
                (Key::X, button(ConsoleButton::Select)),
                (Key::Semicolon, Binding::Console(ConsoleBinding::OpenLid)),
                (Key::Slash, Binding::Console(ConsoleBinding::CloseLid)),
                (Key::Comma, Binding::Command(FrontendCommand::PlayPause)),
                (Key::Period, Binding::Command(FrontendCommand::Step)),
            ]
            .into_iter()
            .map(|basic| {
                (
                    KeyCombination {
                        key_code: basic.0,
                        modifiers: Modifiers::empty(),
                    },
                    basic.1,
                )
            })
            .chain(vec![(
                KeyCombination {
                    key_code: Key::S,
                    modifiers: Modifiers::CTRL,
                },
                Binding::Command(FrontendCommand::WriteSavedata(String::from("save.bin"))),
            )])
            .collect(),
        }
    }
}

fn button(button: ConsoleButton) -> Binding {
    Binding::Console(ConsoleBinding::Button(button))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every binding must have a YAML spelling. serde_yaml refuses to serialize
    /// nested enums, which is why `ConfigBinding` is flat.
    #[test]
    fn the_default_config_survives_a_yaml_round_trip() {
        let file = ConfigFile::from(Config::default());
        let yaml = serde_yaml::to_string(&file).expect("default config is not serializable");

        assert_eq!(
            serde_yaml::from_str::<ConfigFile>(&yaml).expect("serialized config does not parse"),
            file
        );
    }

    #[test]
    fn the_shipped_config_file_loads() {
        let yaml = std::fs::read_to_string("config.yml").expect("config.yml is missing");

        serde_yaml::from_str::<ConfigFile>(&yaml).expect("config.yml does not parse");
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    pub key_code: Key,
    pub modifiers: Option<Modifiers>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigKeyMapEntry {
    key: KeyEntry,
    binding: ConfigBinding,
}

/// The file-facing spelling of a [`Binding`].
///
/// Flat by necessity: serde_yaml cannot represent nested enums, so
/// `Console(Button(A))` has no YAML form. Keeping the flat version here also
/// means the file format can change without disturbing the input module.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ConfigBinding {
    Button(ConsoleButton),
    OpenLid,
    CloseLid,
    PlayPause,
    Step,
    WriteSavedata(String),
    ReadSavestate(String),
    WriteSavestate(String),
    WriteMainRam(String),
    ToggleReplayMode,
    SaveReplay,
}

impl From<ConfigBinding> for Binding {
    fn from(value: ConfigBinding) -> Self {
        match value {
            ConfigBinding::Button(button) => Binding::Console(ConsoleBinding::Button(button)),
            ConfigBinding::OpenLid => Binding::Console(ConsoleBinding::OpenLid),
            ConfigBinding::CloseLid => Binding::Console(ConsoleBinding::CloseLid),
            ConfigBinding::PlayPause => Binding::Command(FrontendCommand::PlayPause),
            ConfigBinding::Step => Binding::Command(FrontendCommand::Step),
            ConfigBinding::WriteSavedata(path) => {
                Binding::Command(FrontendCommand::WriteSavedata(path))
            }
            ConfigBinding::ReadSavestate(path) => {
                Binding::Command(FrontendCommand::ReadSavestate(path))
            }
            ConfigBinding::WriteSavestate(path) => {
                Binding::Command(FrontendCommand::WriteSavestate(path))
            }
            ConfigBinding::WriteMainRam(path) => {
                Binding::Command(FrontendCommand::WriteMainRam(path))
            }
            ConfigBinding::ToggleReplayMode => Binding::Command(FrontendCommand::ToggleReplayMode),
            ConfigBinding::SaveReplay => Binding::Command(FrontendCommand::SaveReplay),
        }
    }
}

impl From<Binding> for ConfigBinding {
    fn from(value: Binding) -> Self {
        match value {
            Binding::Console(ConsoleBinding::Button(button)) => ConfigBinding::Button(button),
            Binding::Console(ConsoleBinding::OpenLid) => ConfigBinding::OpenLid,
            Binding::Console(ConsoleBinding::CloseLid) => ConfigBinding::CloseLid,
            Binding::Command(FrontendCommand::PlayPause) => ConfigBinding::PlayPause,
            Binding::Command(FrontendCommand::Step) => ConfigBinding::Step,
            Binding::Command(FrontendCommand::WriteSavedata(path)) => {
                ConfigBinding::WriteSavedata(path)
            }
            Binding::Command(FrontendCommand::ReadSavestate(path)) => {
                ConfigBinding::ReadSavestate(path)
            }
            Binding::Command(FrontendCommand::WriteSavestate(path)) => {
                ConfigBinding::WriteSavestate(path)
            }
            Binding::Command(FrontendCommand::WriteMainRam(path)) => {
                ConfigBinding::WriteMainRam(path)
            }
            Binding::Command(FrontendCommand::ToggleReplayMode) => ConfigBinding::ToggleReplayMode,
            Binding::Command(FrontendCommand::SaveReplay) => ConfigBinding::SaveReplay,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub default_game_path: Option<PathBuf>,
    pub default_save_path: Option<PathBuf>,
    pub timestamp: Option<DateTime<Utc>>,
    pub key_map: Vec<ConfigKeyMapEntry>,
}

impl From<KeyEntry> for KeyCombination {
    fn from(value: KeyEntry) -> Self {
        KeyCombination {
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
                .map(|entry| (entry.key.into(), entry.binding.into()))
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
                .map(|(key, binding)| ConfigKeyMapEntry {
                    key: key.into(),
                    binding: binding.into(),
                })
                .collect(),
        }
    }
}

impl From<KeyCombination> for KeyEntry {
    fn from(value: KeyCombination) -> Self {
        KeyEntry {
            key_code: value.key_code,
            modifiers: if value.modifiers.eq(&Default::default()) {
                None
            } else {
                Some(value.modifiers)
            },
        }
    }
}
