use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use clap::Parser;
use glium::glutin::{self, event::Event};
use tokio::time as ttime;

use crate::args::Commands;
use crate::audio::Audio;
use crate::config::{Config, ConfigFile};
use crate::frontend::{Frontend, ReplayState};
use crate::game_thread::GameThread;
use crate::replay::Replay;
use crate::replay::ReplaySource;
use crate::window::{draw, get_draw_data};

pub mod args;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod game_thread;
pub mod melon;
pub mod replay;
pub mod utils;
pub mod window;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    let config: Config = std::fs::read_to_string("config.yml")
        .ok()
        .map(|yml| serde_yaml::from_str::<ConfigFile>(&yml).unwrap())
        .map(Into::into)
        .unwrap_or_default();
    let game_name = args.game.as_ref().or(config.default_game_path.as_ref()).map(Clone::clone).expect("No game was selected in the command arguments, and no default game was included in the config");
    let mut save_name = None;
    let mut replay: Option<(Replay, ReplayState)> = None;

    let mut start_time = config.timestamp.unwrap_or_else(Utc::now);

    match &args.command {
        Commands::Play(play_args) => {
            if !play_args.no_save {
                save_name = play_args
                    .save
                    .as_ref()
                    .or(config.default_save_path.as_ref())
                    .map(Clone::clone);
            }
        }
        Commands::Replay(replay_args) => {
            replay = Some((
                serde_yaml::from_str(&std::fs::read_to_string(&replay_args.name).unwrap()).unwrap(),
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

    let cart = std::fs::read(&game_name).unwrap_or_else(|_| {
        panic!(
            "Couldn't find game file with path {}",
            game_name.to_string_lossy()
        )
    });
    let save = save_name.map(|name| {
        std::fs::read(&name).unwrap_or_else(|_| {
            panic!(
                "Couldn't open save file with path {}",
                name.to_string_lossy()
            )
        })
    });
    println!("start_time = {}", start_time);
    let mut audio = Audio::new();

    let frontend = Frontend::new(audio.get_game_stream(), config.key_map, replay);
    let emu = Arc::new(Mutex::new(frontend));

    let mut game_thread = GameThread::new(emu.clone(), cart, save, start_time);
    let mut game_thread_handle = Some(tokio::spawn(async move {
        let mut timer = ttime::interval_at(
            ttime::Instant::now(),
            ttime::Duration::from_nanos(16_666_667),
        );
        timer.set_missed_tick_behavior(ttime::MissedTickBehavior::Skip);
        loop {
            timer.tick().await;

            game_thread.execute()
        }
    }));

    let events_loop = glutin::event_loop::EventLoop::new();
    let display = glium::Display::new(
        glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(256.0, 192.0 * 2.0))
            .with_title("melon-rs"),
        glutin::ContextBuilder::new(),
        &events_loop,
    )
    .unwrap();

    let draw_data = get_draw_data(&display);

    events_loop.run(move |ev, _target, control_flow| {
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_secs_f64(1.0 / 60.0);

        let (top_frame, bottom_frame) = {
            let emu_lock = (*emu).lock().unwrap();
            (emu_lock.top_frame, emu_lock.bottom_frame)
        };

        draw(&display, &draw_data, &top_frame, &bottom_frame);

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        if let Event::WindowEvent { event, .. } = ev {
            let escape = emu.lock().unwrap().window_event(event);
            if escape {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
                game_thread_handle.take();
            }
        }
    });
}
