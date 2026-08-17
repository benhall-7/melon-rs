use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, watch};

use crate::app::{App, RepaintHandle, native_options};
use crate::audio::Playback;
use crate::config::Config;
use crate::frontend::{Frames, Frontend, ReplayState, Request, Save};
use crate::input::{Binding, InputBridge, InputEvent, KeyCombination};
use crate::observe::FrameObserver;
use crate::render::{RenderHook, RenderStatus};
use crate::replay::Replay;
use crate::{EmuState, EmuStateChange};

/// Everything needed to start the emulator after ROM and save bytes are loaded.
pub struct RunParams {
    pub cart: Vec<u8>,
    pub save: Option<Vec<u8>>,
    pub start_time: DateTime<Utc>,
    pub replay: Option<(Replay, ReplayState)>,
    pub key_map: HashMap<KeyCombination, Binding>,
    pub window_title: String,
}

impl RunParams {
    pub fn new(cart: Vec<u8>) -> Self {
        Self {
            cart,
            save: None,
            start_time: Utc::now(),
            replay: None,
            key_map: Config::default().key_map,
            window_title: String::from("melon-rs"),
        }
    }
}

/// Opens the window, runs emulation until it closes, then shuts down cleanly.
pub fn run(
    params: RunParams,
    observers: impl IntoIterator<Item = Box<dyn FrameObserver>>,
    render_hooks: impl IntoIterator<Item = Box<dyn RenderHook>>,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("failed to build the main runtime");

    runtime.block_on(async move {
        println!("start_time = {}", params.start_time);

        let (_playback, audio) = Playback::start();

        let (input_tx, input_rx) = mpsc::channel::<InputEvent>(128);
        let (input_bridge, input_wake_rx) = InputBridge::new(input_tx);
        let (request_tx, request_rx) = mpsc::channel::<Request>(16);
        let (state_tx, state_rx) = watch::channel(None);
        let (save_tx, save_rx) = mpsc::channel::<Save>(8);
        let (frames_tx, frames_rx) = watch::channel(Arc::new(Frames::blank()));
        let (status_tx, status_rx) =
            watch::channel(RenderStatus {
                frame: 0,
                paused: true,
            });

        thread::Builder::new()
            .name("file-saver".to_owned())
            .spawn(move || write_saves(save_rx))
            .expect("failed to spawn the save writer thread");

        let repaint: RepaintHandle = Arc::new(OnceLock::new());
        let render_hooks: Vec<Box<dyn RenderHook>> = render_hooks.into_iter().collect();

        let emulator = Emulator {
            frontend: Frontend::new(
                params.cart,
                params.save,
                params.start_time,
                audio,
                params.key_map,
                params.replay,
                frames_tx,
            )
            .with_observers(observers),
            state: EmuState::Paused,
            status_tx,
            state_tx: state_tx.clone(),
            state_rx,
            request_tx,
            request_rx,
            input_rx,
            input_wake: input_wake_rx,
            saves: save_tx,
            repaint: repaint.clone(),
        };

        let thread = thread::Builder::new()
            .name("emulator".to_owned())
            .spawn(move || emulator.run())
            .expect("failed to spawn the emulator thread");

        let window_state_tx = state_tx.clone();
        let window_title = params.window_title;

        eframe::run_native(
            &window_title,
            native_options(),
            Box::new(move |cc| {
                Ok(Box::new(App::new(
                    cc,
                    frames_rx,
                    status_rx,
                    render_hooks,
                    input_bridge,
                    window_state_tx,
                    &repaint,
                )))
            }),
        )
        .expect("failed to open the window");

        let _ = state_tx.send(Some(EmuStateChange::Stop));
        let _ = thread.join();
    });
}

struct Emulator {
    frontend: Frontend,
    state: EmuState,
    status_tx: watch::Sender<RenderStatus>,
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
        // Stop is sent from the UI thread on window close; do not rely on
        // has_changed alone, in case we were busy when it arrived.
        if *self.state_rx.borrow() == Some(EmuStateChange::Stop) {
            self.state = EmuState::Stopped;
            let _ = self.state_rx.borrow_and_update();
            self.publish_status();
            return;
        }

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

        self.publish_status();
    }

    fn publish_status(&self) {
        let _ = self.status_tx.send(RenderStatus {
            frame: self.frontend.nds.current_frame() as u64,
            paused: !matches!(self.state, EmuState::Running),
        });
    }

    fn drain_input(&mut self) {
        while let Ok(event) = self.input_rx.try_recv() {
            self.frontend
                .handle_input_event(event, &self.state_tx, &self.request_tx);
        }
    }

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
        self.publish_status();

        if let Some(ctx) = self.repaint.get() {
            ctx.request_repaint();
        }
    }
}

fn write_saves(mut saves: mpsc::Receiver<Save>) {
    while let Some(save) = saves.blocking_recv() {
        let result = save
            .path
            .parent()
            .map(std::fs::create_dir_all)
            .transpose()
            .and_then(|_| std::fs::write(&save.path, &save.contents));

        match result {
            Ok(()) => println!("wrote {}", save.path.display()),
            Err(err) => println!("WARNING: couldn't write {}: {err}", save.path.display()),
        }
    }
}
