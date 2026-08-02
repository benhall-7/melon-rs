use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use clap::Parser;
use tokio::sync::{mpsc, watch};

use crate::app::{App, RepaintHandle};
use crate::audio::Playback;
use crate::config::{Config, ConfigFile, StartParams};
use crate::frontend::{Frontend, Request};
use crate::input::InputEvent;

pub mod app;
pub mod args;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod input;
pub mod melon;
pub mod replay;
pub mod utils;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Running,
    Paused,
    Stepping,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub enum EmuStateChange {
    PlayPause,
    Step,
    Stop,
}

#[derive(Clone)]
pub struct EmulatorHandle {
    pub core: Arc<Mutex<Frontend>>,
    pub state: EmuState,
    pub input_tx: mpsc::Sender<InputEvent>,
    pub request_tx: mpsc::Sender<Request>,
    pub state_tx: watch::Sender<Option<EmuStateChange>>,
}

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    let config: Config = std::fs::read_to_string("config.yml")
        .ok()
        .map(|yml| serde_yaml::from_str::<ConfigFile>(&yml).unwrap())
        .map(Into::into)
        .unwrap_or_default();

    let StartParams {
        replay,
        game_name,
        save_name,
        start_time,
    } = config.get_start_params(args);

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
    // Playback stops the moment this is dropped, so it lives as long as main.
    let (_playback, audio) = Playback::start();

    let core = Arc::new(Mutex::new(Frontend::new(
        cart,
        save,
        start_time,
        audio,
        config.key_map,
        replay,
    )));

    // TODO: figure out the ownership model for all the receivers, senders, and emulator handle
    let (input_tx, mut input_rx) = mpsc::channel::<InputEvent>(128);
    let (request_tx, mut request_rx) = mpsc::channel::<Request>(16);
    let (state_tx, mut state_rx) = watch::channel(None);

    let mut emulator = EmulatorHandle {
        core: core.clone(),
        state: EmuState::Paused,
        input_tx: input_tx.clone(),
        request_tx: request_tx.clone(),
        state_tx: state_tx.clone(),
    };

    let repaint: RepaintHandle = Arc::new(OnceLock::new());
    let emulator_repaint = repaint.clone();

    // Spawn emulator tick loop
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_nanos(16_666_667));
        loop {
            timer.tick().await;

            if state_rx.has_changed().unwrap() {
                let state_change = *state_rx.borrow_and_update();
                match state_change {
                    Some(EmuStateChange::PlayPause) => match emulator.state {
                        EmuState::Running => emulator.state = EmuState::Paused,
                        EmuState::Paused => emulator.state = EmuState::Running,
                        EmuState::Stepping => emulator.state = EmuState::Running,
                        EmuState::Stopped => {}
                    },
                    Some(EmuStateChange::Step) => match emulator.state {
                        EmuState::Running => emulator.state = EmuState::Stepping,
                        EmuState::Paused => emulator.state = EmuState::Stepping,
                        EmuState::Stepping => emulator.state = EmuState::Stepping,
                        EmuState::Stopped => {}
                    },
                    Some(EmuStateChange::Stop) => emulator.state = EmuState::Stopped,
                    _ => {}
                }
            }

            while let Ok(req) = request_rx.try_recv() {
                let mut guard = core.lock().unwrap();
                match req {
                    Request::WriteRam(path_buf) => {
                        let ram = guard.nds.main_ram();
                        std::fs::write(&path_buf, ram).unwrap();
                        println!("main RAM written to {}", path_buf.display());
                    }
                    Request::WriteSavedata(path_buf) => {
                        let savedata = guard.nds.save_data();
                        std::fs::write(&path_buf, savedata).unwrap();
                        println!("savedata written to {}", path_buf.display());
                    }
                    Request::WriteSavestate(path_buf) => {
                        guard.write_savestate(path_buf.to_string_lossy().into_owned());
                        println!("savestate written to {}", path_buf.display());
                    }
                    Request::ReadSavestate(path_buf) => {
                        guard.read_savestate(path_buf.to_string_lossy().into_owned());
                        println!("savestate read from {}", path_buf.display());
                    }
                    Request::WriteReplay => {
                        if let Some(replay) = &guard.replay {
                            let file = replay.0.name.clone();
                            std::fs::write(file, serde_yaml::to_string(&replay.0).unwrap())
                                .unwrap();
                            println!("replay written to {}", replay.0.name.to_string_lossy());
                        }
                    }
                }
            }

            match emulator.state {
                EmuState::Running => {
                    update_inputs(&core, &mut input_rx, &state_tx, &request_tx);
                    tick_emulator(&core, &emulator_repaint);
                }
                EmuState::Paused => {
                    update_inputs(&core, &mut input_rx, &state_tx, &request_tx);
                }
                EmuState::Stepping => {
                    update_inputs(&core, &mut input_rx, &state_tx, &request_tx);
                    tick_emulator(&core, &emulator_repaint);
                    emulator.state = EmuState::Paused;
                }
                EmuState::Stopped => break,
            }
        }
    });

    eframe::run_native(
        "melon-rs",
        app::native_options(),
        Box::new(move |cc| {
            Ok(Box::new(App::new(
                cc,
                emulator.core.clone(),
                input_tx,
                emulator.state_tx.clone(),
                &repaint,
            )))
        }),
    )
    .expect("failed to open the window");
}

fn update_inputs(
    core: &Arc<Mutex<Frontend>>,
    input_rx: &mut mpsc::Receiver<InputEvent>,
    state_tx: &watch::Sender<Option<EmuStateChange>>,
    request_tx: &mpsc::Sender<Request>,
) {
    while let Ok(event) = input_rx.try_recv() {
        core.lock()
            .map(|mut core| core.handle_input_event(event, state_tx, request_tx))
            .expect("failed to access core lock");
    }
}

fn tick_emulator(core: &Arc<Mutex<Frontend>>, repaint: &RepaintHandle) {
    core.lock()
        .map(|mut core| core.run_frame())
        .expect("failed to access core lock");

    // The UI only draws on demand, so a finished frame has to ask for it. Doing
    // it here rather than on a timer keeps repaint throttling from ever
    // affecting emulation.
    if let Some(ctx) = repaint.get() {
        ctx.request_repaint();
    }
}
