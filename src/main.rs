use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser;
use glium::glutin::{self, event::Event};
use tokio::sync::{mpsc, watch};
use winit::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::ControlFlow;

use crate::audio::Audio;
use crate::config::{Config, ConfigFile, StartParams};
use crate::frontend::{
    EmuInput, EmuState, Frontend, InputEvent, KeyPressAction, ReplayState, Request,
};
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

#[derive(Clone)]
pub struct EmulatorHandle {
    pub core: Arc<Mutex<Frontend>>,
    pub input_tx: mpsc::Sender<InputEvent>,
    pub request_tx: mpsc::Sender<Request>,
    pub state_tx: watch::Sender<EmuState>,
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

    let (input_tx, mut input_rx) = mpsc::channel::<InputEvent>(128);
    let (request_tx, request_rx) = mpsc::channel::<Request>(16);
    let (state_tx, state_rx) = watch::channel(EmuState::Run);

    let emulator = EmulatorHandle {
        core: core.clone(),
        input_tx: input_tx.clone(),
        request_tx: request_tx.clone(),
        state_tx: state_tx.clone(),
    };

    // Spawn emulator tick loop
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_nanos(16_666_667));
        loop {
            timer.tick().await;

            if *state_rx.borrow() == EmuState::Run {
                tick_emulator(&core, &mut input_rx, &state_tx).await;
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
                        ElementState::Released => input_tx.try_send(InputEvent::MouseDown).unwrap(),
                    },
                    _ => {}
                },
                WindowEvent::CursorMoved { position, .. } => {
                    println!("position: {},{}", position.x, position.y);
                }
                WindowEvent::CloseRequested => {
                    emulator.state_tx.send(EmuState::Stop).unwrap();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
    });
}

async fn tick_emulator(
    core: &Arc<Mutex<Frontend>>,
    input_rx: &mut mpsc::Receiver<InputEvent>,
    state_tx: &watch::Sender<EmuState>,
) {
    // Apply input events
    while let Ok(event) = input_rx.try_recv() {
        core.lock()
            .map(|mut core| core.handle_input_event(event))
            .expect("failed to access core lock");
    }

    core.lock()
        .map(|mut core| core.run_frame())
        .expect("failed to access core lock");
}

// fn do_requests_old() {
//     self.emu
//         .lock()
//         .map(|mut emu| {
//             if let Some(read_path) = emu.requests.savestate_read_request.take() {
//                 emu.read_savestate(&mut self.ds, read_path);
//             }
//         })
//         .unwrap();

//     self.emu
//         .lock()
//         .map(|mut emu| {
//             if let Some(write_path) = emu.requests.savestate_write_request.take() {
//                 emu.write_savestate(&mut self.ds, write_path);
//             }
//         })
//         .unwrap();

//     self.emu
//         .lock()
//         .map(|mut emu| {
//             if let Some(write_path) = emu.requests.ram_write_request.take() {
//                 let ram = self.ds.main_ram();
//                 std::fs::write(write_path, ram).unwrap();
//                 println!("main RAM written to ram.bin");
//             }
//         })
//         .unwrap();

//     self.emu
//         .lock()
//         .map(|mut emu| {
//             if emu.requests.replay_save_request {
//                 emu.requests.replay_save_request = false;

//                 if let Some(replay) = &emu.replay {
//                     let file = replay.0.name.clone();
//                     std::fs::write(file, serde_yaml::to_string(&replay.0).unwrap()).unwrap();
//                     println!("saved replay to {}", replay.0.name.to_string_lossy());
//                 }
//             }
//         })
//         .unwrap();
// }
