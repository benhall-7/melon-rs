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
use crate::frontend::{EmuInput, EmuState, KeyPressAction, ReplayState};
use crate::melon::nds::input::NdsKeyMask;
use crate::melon::nds::Nds;
use crate::replay::Replay;
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

// move to own file
enum InputEvent {
    KeyDown(VirtualKeyCode),
    KeyUp(VirtualKeyCode),
    CursorMove(Option<(u8, u8)>),
    MouseDown,
    MouseUp,
    KeyModifierChange(ModifiersState),
}

enum Request {
    SavestateWrite(PathBuf),
    SavestateRead(PathBuf),
    RamWrite(PathBuf),
    ReplaySave,
}

// #[derive(Debug)]
pub struct FrontendCore {
    pub nds: Nds,
    pub top_frame: [u8; 256 * 192 * 4],
    pub bottom_frame: [u8; 256 * 192 * 4],
    pub audio: Arc<Mutex<Vec<i16>>>,
    pub key_map: HashMap<EmuInput, KeyPressAction>,
    pub key_modifiers: ModifiersState,
    pub nds_input: NdsKeyMask,
    pub cursor: Option<(u8, u8)>,
    pub cursor_pressed: bool,
    pub state: EmuState,
    pub replay: Option<(Replay, ReplayState)>,
}

#[derive(Clone)]
pub struct EmulatorHandle {
    pub core: Arc<Mutex<FrontendCore>>,
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

    // NDS initialization
    let mut nds = Nds::new();

    nds.set_nds_cart(&cart, save.as_deref());
    nds.set_time(start_time);

    println!("Needs direct boot? {:?}", nds.needs_direct_boot());

    if nds.needs_direct_boot() {
        nds.setup_direct_boot(String::from("Ultra.nds"));
    }

    nds.start();

    let core = Arc::new(Mutex::new(FrontendCore {
        nds,
        top_frame: [0; 256 * 192 * 4],
        bottom_frame: [0; 256 * 192 * 4],
        audio: audio.get_game_stream(),
        key_map: config.key_map,
        key_modifiers: ModifiersState::empty(),
        nds_input: NdsKeyMask::empty(),
        cursor: None,
        cursor_pressed: false,
        state: EmuState::Pause,
        replay,
    }));

    let (input_tx, mut input_rx) = mpsc::channel::<InputEvent>(128);
    let (request_tx, mut request_rx) = mpsc::channel::<Request>(16);
    let (state_tx, mut state_rx) = watch::channel(EmuState::Run);

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

            // Optionally skip ticking if paused
            if *state_rx.borrow() != EmuState::Run {
                continue;
            }

            tick_emulator(&core, &mut input_rx, &request_tx).await;
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
                WindowEvent::MouseInput {
                    device_id: _,
                    state,
                    button,
                    modifiers: _,
                } => match button {
                    MouseButton::Left => match state {
                        ElementState::Pressed => input_tx.try_send(InputEvent::MouseDown).unwrap(),
                        ElementState::Released => input_tx.try_send(InputEvent::MouseDown).unwrap(),
                    },
                    _ => {}
                },
                WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } => {
                    println!("position: {},{}", position.x, position.y);
                }
                WindowEvent::CloseRequested => {
                    state_tx.send(EmuState::Stop).unwrap();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
    });
}

async fn tick_emulator(
    core: &Arc<Mutex<FrontendCore>>,
    input_rx: &mut mpsc::Receiver<InputEvent>,
    request_tx: &mpsc::Sender<Request>,
) {
    // Apply input events
    while let Ok(event) = input_rx.try_recv() {
        let mut guard = core.lock().unwrap();
        match event {
            InputEvent::KeyDown(key_code) => {
                let modifiers = guard.key_modifiers;

                let emu_key = match guard.key_map.get(&EmuInput {
                    key_code,
                    modifiers,
                }) {
                    Some(action) => action.to_owned(),
                    None => return,
                };

                match emu_key {
                    KeyPressAction::NdsKey(nds_key) => guard.nds_input.insert(nds_key.into()),
                    KeyPressAction::PlayPlause => {
                        let emu_state = &mut guard.state;
                        match *emu_state {
                            EmuState::Pause | EmuState::Step => *emu_state = EmuState::Run,
                            EmuState::Run => *emu_state = EmuState::Pause,
                            EmuState::Stop => {}
                        }
                    }
                    KeyPressAction::Step => {
                        let emu_state = &mut guard.state;
                        match *emu_state {
                            EmuState::Stop => {}
                            _ => *emu_state = EmuState::Step,
                        }
                    }
                    // TODO: implement these actions
                    KeyPressAction::Save(path) => {
                        // ...
                    }
                    KeyPressAction::ReadSavestate(path) => {
                        // self.requests.savestate_read_request = Some(path);
                    }
                    KeyPressAction::WriteSavestate(path) => {
                        // self.requests.savestate_write_request = Some(path);
                    }
                    KeyPressAction::ToggleReplayMode => {
                        if let Some(state) = guard.replay.as_mut() {
                            match state.1 {
                                ReplayState::Playing => {
                                    state.1 = ReplayState::Recording;
                                    println!("Switched to write mode");
                                }
                                ReplayState::Recording => {
                                    state.1 = ReplayState::Playing;
                                    println!("Switched to read mode");
                                }
                            }
                        }
                    }
                    KeyPressAction::SaveReplay => {
                        // self.requests.replay_save_request = true;
                    }
                    KeyPressAction::WriteMainRAM(path) => {
                        // self.requests.ram_write_request = Some(path);
                    }
                    _ => {}
                }
            }
            InputEvent::KeyUp(key_code) => {
                let modifiers = guard.key_modifiers;

                let emu_key = match guard.key_map.get(&EmuInput {
                    key_code,
                    modifiers,
                }) {
                    Some(action) => action.to_owned(),
                    None => return,
                };

                match emu_key {
                    KeyPressAction::NdsKey(nds_key) => guard.nds_input.remove(nds_key.into()),
                    _ => {}
                }
            }
            InputEvent::CursorMove(coord) => guard.cursor = coord,
            InputEvent::MouseDown => guard.cursor_pressed = true,
            InputEvent::MouseUp => guard.cursor_pressed = false,
            InputEvent::KeyModifierChange(mods) => guard.key_modifiers = mods,
        }
    }

    {
        let mut guard = core.lock().unwrap();
        if guard.state != EmuState::Run {
            return;
        }

        // set inputs before frame
        let key_mask = guard.nds_input;
        guard.nds.set_key_mask(key_mask);
        guard.nds.run_frame();

        // audio
        let audio_out = guard.nds.read_audio_output();
        guard
            .audio
            .lock()
            .map(|mut stream| stream.extend(audio_out))
            .expect("failed to access audio lock");

        // frames
        let mut top = guard.top_frame;
        let mut bottom = guard.bottom_frame;
        guard.nds.update_framebuffers(&mut top, false);
        guard.nds.update_framebuffers(&mut bottom, true);
        guard.top_frame = top;
        guard.bottom_frame = bottom;
    }
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
