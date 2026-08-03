use std::sync::{Arc, OnceLock};
use std::time::Duration;

use clap::Parser;
use tokio::sync::{mpsc, watch};

use crate::app::{App, RepaintHandle};
use crate::audio::Playback;
use crate::config::{Config, ConfigFile, StartParams};
use crate::frontend::{Frames, Frontend, Request, Save};
use crate::input::{InputBridge, InputEvent};

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

/// The emulator and everything it talks to.
struct Emulator {
    frontend: Frontend,
    state: EmuState,
    /// Bindings raise state changes and requests, which arrive back here on a later tick.
    state_tx: watch::Sender<Option<EmuStateChange>>,
    state_rx: watch::Receiver<Option<EmuStateChange>>,
    request_tx: mpsc::Sender<Request>,
    request_rx: mpsc::Receiver<Request>,
    input_rx: mpsc::Receiver<InputEvent>,
    input_wake: watch::Receiver<u64>,
    saves: mpsc::Sender<Save>,
    repaint: RepaintHandle,
}

impl Emulator {
    const FRAME: Duration = Duration::from_nanos(16_666_667);

    /// Runs frames on their own clock until told to stop.
    ///
    /// This is a blocking, fixed-cadence job, so it gets a thread and a runtime
    /// to itself: nothing else competes for the wake, and `interval` schedules
    /// against a fixed origin, which sleeping one period per iteration cannot do
    /// without accumulating every overshoot.
    fn run(mut self) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("failed to build the emulator runtime");

        runtime.block_on(async move {
            let mut timer = tokio::time::interval(Self::FRAME);

            loop {
                tokio::select! {
                    _ = self.input_wake.changed() => {
                        let _ = self.input_wake.borrow_and_update();
                        self.drain_input();
                        self.apply_state_change();
                    }

                    _ = timer.tick() => {
                        self.apply_state_change();
                        self.serve_requests();
                        self.drain_input();

                        match self.state {
                            EmuState::Running => self.tick(),
                            EmuState::Paused => {}
                            EmuState::Stepping => {
                                self.tick();
                                self.state = EmuState::Paused;
                            }
                            EmuState::Stopped => {}
                        }
                    }
                }

                if self.state == EmuState::Stopped {
                    break;
                }
            }
        });
    }

    fn apply_state_change(&mut self) {
        if !self.state_rx.has_changed().unwrap_or(false) {
            return;
        }

        let change = *self.state_rx.borrow_and_update();

        self.state = match (change, self.state) {
            (Some(EmuStateChange::Stop), _) => EmuState::Stopped,
            (Some(EmuStateChange::PlayPause), EmuState::Running) => EmuState::Paused,
            (Some(EmuStateChange::PlayPause), EmuState::Paused | EmuState::Stepping) => {
                EmuState::Running
            }
            (Some(EmuStateChange::Step), EmuState::Running | EmuState::Paused) => {
                EmuState::Stepping
            }
            (_, state) => state,
        };
    }

    fn drain_input(&mut self) {
        while let Ok(event) = self.input_rx.try_recv() {
            self.frontend
                .handle_input_event(event, &self.state_tx, &self.request_tx);
        }
    }

    /// Turns requests into bytes and hands the writing to someone else.
    fn serve_requests(&mut self) {
        while let Ok(request) = self.request_rx.try_recv() {
            let saves = match request {
                Request::WriteRam(path) => vec![Save {
                    path,
                    contents: self.frontend.nds.main_ram().to_vec(),
                }],
                Request::WriteSavedata(path) => vec![Save {
                    path,
                    contents: self.frontend.nds.save_data().to_vec(),
                }],
                Request::WriteSavestate(path) => {
                    self.frontend.savestate(path.to_string_lossy().into_owned())
                }
                Request::WriteReplay => self.frontend.replay_save().into_iter().collect(),
                // Loading mutates the console, so it cannot leave this thread.
                Request::ReadSavestate(path) => {
                    self.frontend
                        .read_savestate(path.to_string_lossy().into_owned());
                    continue;
                }
            };

            for save in saves {
                if let Err(err) = self.saves.try_send(save) {
                    println!("WARNING: a file was not written: {err}");
                }
            }
        }
    }

    fn tick(&mut self) {
        self.frontend.run_frame();

        // The UI only draws on demand, so a finished frame has to ask for it.
        // Doing it here rather than on a timer keeps repaint throttling from
        // ever affecting emulation.
        if let Some(ctx) = self.repaint.get() {
            ctx.request_repaint();
        }
    }
}

/// Writes what the emulator hands over, so that it never waits on a disk.
async fn write_saves(mut saves: mpsc::Receiver<Save>) {
    while let Some(save) = saves.recv().await {
        match tokio::fs::write(&save.path, &save.contents).await {
            Ok(()) => println!("wrote {}", save.path.display()),
            Err(err) => println!("WARNING: couldn't write {}: {err}", save.path.display()),
        }
    }
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

    let (input_tx, input_rx) = mpsc::channel::<InputEvent>(128);
    let (input_bridge, input_wake_rx) = InputBridge::new(input_tx);
    let (request_tx, request_rx) = mpsc::channel::<Request>(16);
    let (state_tx, state_rx) = watch::channel(None);
    let (save_tx, save_rx) = mpsc::channel::<Save>(8);
    let (frames_tx, frames_rx) = watch::channel(Arc::new(Frames::blank()));

    tokio::spawn(write_saves(save_rx));

    let repaint: RepaintHandle = Arc::new(OnceLock::new());

    let emulator = Emulator {
        frontend: Frontend::new(
            cart,
            save,
            start_time,
            audio,
            config.key_map,
            replay,
            frames_tx,
        ),
        state: EmuState::Paused,
        state_tx: state_tx.clone(),
        state_rx,
        request_tx,
        request_rx,
        input_rx,
        input_wake: input_wake_rx,
        saves: save_tx,
        repaint: repaint.clone(),
    };

    let thread = std::thread::Builder::new()
        .name("emulator".to_owned())
        .spawn(move || emulator.run())
        .expect("failed to spawn the emulator thread");

    let window_state_tx = state_tx.clone();

    eframe::run_native(
        "melon-rs",
        app::native_options(),
        Box::new(move |cc| {
            Ok(Box::new(App::new(
                cc,
                frames_rx,
                input_bridge,
                window_state_tx,
                &repaint,
            )))
        }),
    )
    .expect("failed to open the window");

    // The window is gone, so let the emulator finish the frame it is on rather
    // than tearing the console down underneath it.
    let _ = state_tx.send(Some(EmuStateChange::Stop));
    let _ = thread.join();
}
