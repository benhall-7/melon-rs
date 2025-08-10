use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use glium::glutin::{self, event::Event};
use tokio::sync::{mpsc, watch};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::audio::Audio;
use crate::config::{Config, ConfigFile, StartParams};
use crate::frontend::{Frontend, InputEvent, Request};
use crate::window::{draw, get_draw_data};

pub mod args;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod melon;
pub mod replay;
pub mod utils;
pub mod window;

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
    let mut audio = Audio::new();

    let core = Arc::new(Mutex::new(Frontend::new(
        cart,
        save,
        start_time,
        audio.get_game_stream(),
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
                    tick_emulator(&core);
                }
                EmuState::Paused => {
                    update_inputs(&core, &mut input_rx, &state_tx, &request_tx);
                }
                EmuState::Stepping => {
                    update_inputs(&core, &mut input_rx, &state_tx, &request_tx);
                    tick_emulator(&core);
                    emulator.state = EmuState::Paused;
                }
                EmuState::Stopped => break,
            }
        }
    });

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
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        // TODO: double-buffer the frames so that we don't read from the same data being written
        let (top_frame, bottom_frame) = {
            let emu_lock = (*emulator.core).lock().unwrap();
            (emu_lock.top_frame, emu_lock.bottom_frame)
        };

        draw(&display, &draw_data, &top_frame, &bottom_frame);

        if let Event::WindowEvent { event, .. } = ev {
            match event {
                WindowEvent::ModifiersChanged(modifiers) => {
                    input_tx
                        .try_send(InputEvent::KeyModifierChange(modifiers))
                        .unwrap();
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        let event = match input.state {
                            ElementState::Pressed => InputEvent::KeyDown(key),
                            ElementState::Released => InputEvent::KeyUp(key),
                        };
                        input_tx.try_send(event).unwrap();
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => match button {
                    MouseButton::Left => match state {
                        ElementState::Pressed => input_tx.try_send(InputEvent::MouseDown).unwrap(),
                        ElementState::Released => input_tx.try_send(InputEvent::MouseUp).unwrap(),
                    },
                    _ => {}
                },
                WindowEvent::CursorMoved { position, .. } => {
                    if position.y >= 192.0 {
                        // TODO: check if scaling the screen breaks anything
                        let x = (position.x as u32).clamp(0, 255) as u8;
                        let y = (position.y as u32 - 192).clamp(0, 255) as u8;
                        input_tx
                            .try_send(InputEvent::CursorMove(Some((x, y))))
                            .unwrap();
                    }
                }
                WindowEvent::CloseRequested => {
                    emulator.state_tx.send(Some(EmuStateChange::Stop)).unwrap();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
    });
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

fn tick_emulator(core: &Arc<Mutex<Frontend>>) {
    core.lock()
        .map(|mut core| core.run_frame())
        .expect("failed to access core lock");
}
