mod args;

use std::fs;

use args::{Args, Commands};
use chrono::{DateTime, Utc};
use clap::Parser;
use melon_rs::{
    config::{Config, ConfigFile, StartParams},
    frontend::ReplayState,
    replay::{Replay, ReplaySource},
    run::{RunParams, run},
};

#[ignore = "irrefutable_let_patterns"]
fn start_params(config: &Config, args: Args) -> StartParams {
    let game_name = args
        .game
        .as_ref()
        .or(config.default_game_path.as_ref())
        .cloned()
        .expect(
            "No game was selected in the command arguments, and no default game was included in the config",
        );

    let mut save_name = None;
    let mut replay = None;
    let mut start_time = config.timestamp.unwrap_or_else(Utc::now);

    match &args.command {
        Commands::Play(play_args) => {
            if !play_args.no_save {
                save_name = play_args
                    .save
                    .as_ref()
                    .or(config.default_save_path.as_ref())
                    .cloned();
            }
        }
        Commands::Replay(replay_args) => {
            replay = Some((
                serde_yaml::from_str(&fs::read_to_string(&replay_args.name).unwrap()).unwrap(),
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
            ));
        }
    }

    if let Some((replay, _)) = &replay {
        let ReplaySource::SaveFile { path, timestamp } = &replay.source;
        save_name = path.clone();
        start_time = *timestamp;
    }

    StartParams {
        replay,
        game_name,
        save_name,
        start_time,
    }
}

fn main() {
    let args = Args::parse();

    let config: Config = fs::read_to_string("config.yml")
        .ok()
        .map(|yml| serde_yaml::from_str::<ConfigFile>(&yml).unwrap())
        .map(Into::into)
        .unwrap_or_default();

    let StartParams {
        replay,
        game_name,
        save_name,
        start_time,
    } = start_params(&config, args);

    let cart = fs::read(&game_name).unwrap_or_else(|_| {
        panic!(
            "Couldn't find game file with path {}",
            game_name.to_string_lossy()
        )
    });
    let save = save_name.map(|name| {
        fs::read(&name).unwrap_or_else(|_| {
            panic!(
                "Couldn't open save file with path {}",
                name.to_string_lossy()
            )
        })
    });

    run(
        RunParams {
            cart,
            save,
            start_time,
            replay,
            key_map: config.key_map,
            window_title: String::from("melon-rs"),
        },
        vec![],
        vec![],
    );
}
