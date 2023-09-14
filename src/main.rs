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
use tokio::time as ttime;
use window::{draw, get_draw_data};
use winit::event::ModifiersState;

use crate::melon::{nds::input::NdsKeyMask, save};
use crate::replay::ReplaySource;

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
            // replay: Some((
            //     // Replay {
            //     //     name: "MyTAS".into(),
            //     //     author: "Ben Hall".into(),
            //     //     source: replay::ReplaySource::SaveFile {
            //     //         path: "save.bin".into(),
            //     //         timestamp: DateTime::parse_from_str(
            //     //             "2023-09-13T12:00:00+0000",
            //     //             "%Y-%m-%dT%H:%M:%S%.f%z",
            //     //         )
            //     //         .unwrap()
            //     //         .into(),
            //     //     },
            //     //     inputs: vec![],
            //     // },
            //     serde_yaml::from_str(&std::fs::read_to_string("MyTAS").unwrap()).unwrap(),
            //     ReplayState::Playing,
            // )),
            ram_write_request: None,
        }
    }
}

#[tokio::main]
async fn main() {
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
    let mut game_thread = Some(tokio::spawn(async move {
        game(game_emu, game_config).await;
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
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;

                    emu.lock().unwrap().state = EmuState::Stop;
                    game_thread.take().map(|thread| async { thread.await });
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

async fn game(emu: Arc<Mutex<Emu>>, _config: Config) {
    let mut ds_lock = melon::nds::INSTANCE.lock().await;
    let mut ds = ds_lock.take().unwrap();

    let nds_cart = std::fs::read("/Users/benjamin/Desktop/ds/Ultra.nds").unwrap();

    {
        let mut emu = emu.lock().unwrap();
        let replay_opt = emu.replay.as_ref();

        let mut start_time = None;
        let mut save_data = None;

        if let Some((replay, _)) = replay_opt {
            match &replay.source {
                ReplaySource::None { timestamp } => start_time = Some(*timestamp),
                ReplaySource::SaveFile { path, timestamp } => {
                    start_time = Some(*timestamp);
                    save_data = Some(std::fs::read(path).unwrap());
                }
                _ => todo!(),
            }
        } else {
            save_data = std::fs::read("save.bin").ok();
        }

        ds.load_cart(&nds_cart, save_data.as_deref());

        if let Some(timestamp) = start_time {
            emu.time = timestamp;
        }
    }

    println!("Needs direct boot? {:?}", ds.needs_direct_boot());

    if ds.needs_direct_boot() {
        ds.setup_direct_boot(String::from("Ultra.nds"));
    }

    ds.start();

    let mut timer = ttime::interval_at(
        ttime::Instant::now(),
        ttime::Duration::from_nanos(16_666_667),
    );
    timer.set_missed_tick_behavior(ttime::MissedTickBehavior::Skip);
    // let spin_sleeper = spin_sleep::SpinSleeper::new(4_000_000)
    // .with_spin_strategy(spin_sleep::SpinStrategy::YieldThread);
    loop {
        timer.tick().await;

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

        // let elapsed = before_instant.elapsed();

        // if elapsed.as_nanos() < 16_666_667 {
        // spin_sleeper.sleep_ns(16_666_667 - elapsed.as_nanos() as u64);
        // }
    }

    ds.stop();
}
