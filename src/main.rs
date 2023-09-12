use std::sync::{Arc, Mutex};
use std::thread::spawn;

use chrono::{DateTime, Duration, Utc};
use config::{Config, ConfigFile, EmuAction, EmuInput};
use glium::glutin::{
    self,
    event::{ElementState, Event, WindowEvent},
};
use once_cell::sync::Lazy;
use replay::Replay;
use window::{draw, get_draw_data};
use winit::event::ModifiersState;

use crate::melon::{nds::input::NdsKeyMask, save};

pub mod config;
pub mod events;
pub mod melon;
pub mod replay;
pub mod window;

pub static GAME_TIME: Lazy<Arc<Mutex<DateTime<Utc>>>> = Lazy::new(Default::default);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Run,
    Pause,
    Stop,
    Step,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReplayState {
    Recording,
    Playing,
    Write,
}

#[derive(Debug)]
pub struct Emu {
    pub top_frame: [u8; 256 * 192 * 4],
    pub bottom_frame: [u8; 256 * 192 * 4],
    pub nds_input: NdsKeyMask,
    pub key_modifiers: ModifiersState,
    pub state: EmuState,
    pub time: DateTime<Utc>,
    pub savestate_write_request: Option<String>,
    pub savestate_read_request: Option<String>,
    pub replay: Option<(Replay, ReplayState)>,
    pub ram_write_request: Option<String>,
}

impl Emu {
    pub fn new(time: DateTime<Utc>) -> Self {
        Emu {
            top_frame: [0; 256 * 192 * 4],
            bottom_frame: [0; 256 * 192 * 4],
            nds_input: NdsKeyMask::empty(),
            key_modifiers: ModifiersState::empty(),
            time,
            state: EmuState::Pause,
            savestate_read_request: None,
            savestate_write_request: None,
            replay: None,
            ram_write_request: None,
        }
    }
}

fn main() {
    let config: Config = std::fs::read_to_string("config.yml")
        .ok()
        .map(|yml| serde_yaml::from_str::<ConfigFile>(&yml).unwrap())
        .map(Into::into)
        .unwrap_or_default();

    let start_time = config.timestamp.unwrap_or_else(Utc::now);
    println!("start_time = {}", start_time);

    let emu = Arc::new(Mutex::new(Emu::new(start_time)));

    let game_emu = emu.clone();
    let game_config = config.clone();
    let mut game_thread = Some(spawn(|| game(game_emu, game_config)));

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
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;

                    emu.lock().unwrap().state = EmuState::Stop;
                    game_thread.take().map(|thread| thread.join());
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    emu.lock().unwrap().key_modifiers = modifiers;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode.is_none() {
                        return;
                    }
                    let modifiers = emu.lock().unwrap().key_modifiers;

                    let emu_key = match config.key_map.get(&EmuInput {
                        key_code: input.virtual_keycode.unwrap(),
                        modifiers,
                    }) {
                        Some(action) => action.to_owned(),
                        None => return,
                    };

                    let state = input.state;
                    match emu_key {
                        EmuAction::NdsKey(nds_key) => match state {
                            ElementState::Pressed => {
                                emu.lock().unwrap().nds_input.insert(nds_key.into())
                            }
                            ElementState::Released => {
                                emu.lock().unwrap().nds_input.remove(nds_key.into())
                            }
                        },
                        EmuAction::PlayPlause => {
                            if let ElementState::Released = state {
                                return;
                            }
                            let emu_state = &mut emu.lock().unwrap().state;
                            match *emu_state {
                                EmuState::Pause | EmuState::Step => *emu_state = EmuState::Run,
                                EmuState::Run => *emu_state = EmuState::Pause,
                                EmuState::Stop => {}
                            }
                        }
                        EmuAction::Step => {
                            if let ElementState::Released = state {
                                return;
                            }
                            let emu_state = &mut emu.lock().unwrap().state;
                            match *emu_state {
                                EmuState::Stop => {}
                                _ => *emu_state = EmuState::Step,
                            }
                        }
                        EmuAction::Save(path) => {
                            if let ElementState::Released = state {
                                return;
                            }
                            spawn(|| save::update_save(path.into()));
                        }
                        EmuAction::ReadSavestate(path) => {
                            if let ElementState::Released = state {
                                return;
                            }
                            emu.lock().unwrap().savestate_read_request = Some(path);
                        }
                        EmuAction::WriteSavestate(path) => {
                            if let ElementState::Released = state {
                                return;
                            }
                            emu.lock().unwrap().savestate_write_request = Some(path);
                        }
                        EmuAction::QuitRecording => {
                            if let ElementState::Released = state {
                                return;
                            }
                            if let Some(state) = emu.lock().unwrap().replay.as_mut() {
                                state.1 = ReplayState::Write
                            }
                        }
                        EmuAction::WriteMainRAM(path) => {
                            if let ElementState::Released = state {
                                return;
                            }
                            emu.lock().unwrap().ram_write_request = Some(path);
                        }
                    }
                }
                _ => {}
            }
        }
    });
}

fn game(emu: Arc<Mutex<Emu>>, _config: Config) {
    let mut lock = melon::nds::INSTANCE.lock().unwrap();
    let mut ds = lock.take().unwrap();

    ds.load_cart(
        &std::fs::read("/Users/benjamin/Desktop/ds/Ultra.nds").unwrap(),
        std::fs::read("save.bin").ok().as_deref(),
    );

    println!("Needs direct boot? {:?}", ds.needs_direct_boot());

    if ds.needs_direct_boot() {
        ds.setup_direct_boot(String::from("Ultra.nds"));
    }

    ds.start();

    let mut fps = fps_clock::FpsClock::new(60);
    loop {
        let mut force_pause = false;
        let emu_state = emu.lock().unwrap().state;

        {
            let mut lock = emu.lock().unwrap();
            let replay_opt = lock.replay.as_mut();
            if let Some(replay) = replay_opt {
                if let ReplayState::Write = replay.1 {
                    let file = replay.0.name.clone();
                    std::fs::write(file, serde_yaml::to_string(&replay.0).unwrap()).unwrap();
                    lock.replay = None;
                }
            }
        }

        match emu_state {
            EmuState::Stop => break,
            EmuState::Run | EmuState::Step => {
                let nds_key = {
                    let mut lock = emu.lock().unwrap();
                    let emu_inputs = lock.nds_input;
                    match lock.replay.as_mut() {
                        None => emu_inputs,
                        Some((replay, replay_state)) => match *replay_state {
                            ReplayState::Playing => {
                                let current_frame = ds.current_frame() as usize;
                                if current_frame < replay.inputs.len() {
                                    if current_frame == replay.inputs.len() - 1 {
                                        force_pause = true;
                                    }
                                    NdsKeyMask::from_bits_retain(replay.inputs[current_frame])
                                } else {
                                    emu_inputs
                                }
                            }
                            ReplayState::Recording => {
                                let current_frame = ds.current_frame() as usize;
                                if current_frame <= replay.inputs.len() {
                                    replay.inputs.splice(current_frame.., [emu_inputs.bits()]);
                                }
                                // TODO: else block. A frame is skipped in recording (which
                                // can only really happen if you load a bad savestate)
                                emu_inputs
                            }
                            ReplayState::Write => emu_inputs,
                        },
                    }
                };

                // updates static variable for emulator function impl
                // what's a better strategy? maybe the subscriptions?
                *(GAME_TIME.lock().unwrap()) = emu.lock().unwrap().time;

                ds.set_key_mask(nds_key);
                ds.run_frame();

                println!("Frame {}", ds.current_frame());
                println!("Time is now {}", GAME_TIME.lock().unwrap());

                // updates emu time
                emu.lock().unwrap().time += Duration::nanoseconds(16_666_667);

                emu.lock()
                    .map(|mut mutex| {
                        ds.update_framebuffers(&mut mutex.top_frame, false);
                        ds.update_framebuffers(&mut mutex.bottom_frame, true);
                    })
                    .unwrap();

                if force_pause {
                    emu.lock().unwrap().state = EmuState::Pause;
                } else if let EmuState::Step = emu_state {
                    emu.lock().unwrap().state = EmuState::Pause;
                }
            }
            EmuState::Pause => {}
        }

        let (savestate_read_request, savestate_write_request, ram_write_request) = emu
            .lock()
            .map(|mut lock| {
                (
                    lock.savestate_read_request.take(),
                    lock.savestate_write_request.take(),
                    lock.ram_write_request.take(),
                )
            })
            .unwrap();

        if let Some(read_path) = savestate_read_request {
            let (_, time) = ds.read_savestate(read_path);
            emu.lock().unwrap().time = time;
            // it seems this doesn't work, probably because it needs to run
            // a frame to get the right framebuffer data anyway
            // let mut mutex = emu.lock().unwrap();
            // ds.update_framebuffers(&mut mutex.top_frame, false);
            // ds.update_framebuffers(&mut mutex.bottom_frame, true);
        }

        if let Some(write_path) = savestate_write_request {
            let time = emu.lock().unwrap().time;
            ds.write_savestate(write_path, time);
        }

        if let Some(write_path) = ram_write_request {
            let ram = ds.main_ram();
            std::fs::write(write_path, ram).unwrap();
            println!("main RAM written to ram.bin");
        }

        fps.tick();
    }

    ds.stop();
}
