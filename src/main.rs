use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use config::{Config, ConfigFile, EmuAction, EmuInput};
use glium::glutin::{
    self,
    event::{ElementState, Event, WindowEvent},
};
use melon::kssu::addresses::MAIN_RAM_OFFSET;
use melon::kssu::io::MemCursor;
use melon::nds::Nds;
use once_cell::sync::Lazy;
use replay::{Replay, SavestateContext};
use rodio::{OutputStream, Sink};
use tokio::time as ttime;
use utils::localize_pathbuf;
use window::{draw, get_draw_data};
use winit::event::ModifiersState;

use crate::args::Commands;
use crate::melon::kssu::addresses::ACTOR_COLLECTION;
use crate::melon::kssu::{Actor, ActorCollection};
use crate::melon::nds::audio::NdsAudio;
use crate::melon::nds::input::NdsKeyMask;
use crate::melon::save;
use crate::replay::{ReplaySource, SavestateContextReplay};

pub mod args;
pub mod config;
pub mod events;
pub mod melon;
pub mod replay;
pub mod utils;
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
}

// #[derive(Debug)]
pub struct Emu {
    pub top_frame: [u8; 256 * 192 * 4],
    pub bottom_frame: [u8; 256 * 192 * 4],
    // TODO: doesn't implement debug
    pub audio: Sink,
    pub nds_input: NdsKeyMask,
    pub key_modifiers: ModifiersState,
    pub state: EmuState,
    pub time: DateTime<Utc>,
    pub savestate_write_request: Option<String>,
    pub savestate_read_request: Option<String>,
    pub replay: Option<(Replay, ReplayState)>,
    pub ram_write_request: Option<String>,
    pub replay_save_request: bool,
}

impl Emu {
    pub fn new(time: DateTime<Utc>, audio: Sink, replay: Option<(Replay, ReplayState)>) -> Self {
        Emu {
            top_frame: [0; 256 * 192 * 4],
            bottom_frame: [0; 256 * 192 * 4],
            audio,
            nds_input: NdsKeyMask::empty(),
            key_modifiers: ModifiersState::empty(),
            time,
            state: EmuState::Pause,
            savestate_read_request: None,
            savestate_write_request: None,
            replay,
            ram_write_request: None,
            replay_save_request: false,
        }
    }

    // TODO: these may be pretty expensive to run from inside the emu lock
    fn read_savestate(&mut self, nds: &mut Nds, file: String) {
        let localized = localize_pathbuf(file).to_string_lossy().into_owned();

        let mut raw: OsString = localized.clone().into();
        raw.push(".context");
        let context_path = PathBuf::from(raw).to_string_lossy().into_owned();

        let context_str = std::fs::read_to_string(&context_path).ok();
        if context_str.is_none() {
            println!("Couldn't read savestate: {}", context_path);
            return;
        }
        let context_result = serde_yaml::from_str(context_str.as_ref().unwrap());
        if context_result.is_err() {
            println!("Couldn't read savestate context: {}", context_str.unwrap());
            return;
        }
        let context: SavestateContext = context_result.unwrap();

        match (&mut self.replay, context.replay) {
            (Some(replay), Some(replay_context)) => {
                if replay_context.name == replay.0.name {
                    self.time = context.timestamp;
                    replay.0.inputs = replay_context.inputs;
                    assert!(nds.read_savestate(localized));
                } else {
                    println!("The savestate couldn't be loaded. The savestate belongs to a different replay")
                }
            }
            (Some(_), None) => println!("The savestate couldn't be loaded. There is a replay running, but the savestate doesn't belong to one"),
            (None, Some(_)) => println!("The savestate couldn't be loaded. There is no replay running, and the savestate belongs to a replay"),
            (None, None) => {
                self.time = context.timestamp;
                assert!(nds.read_savestate(localized));
            },
        }
    }

    fn write_savestate(&mut self, nds: &mut Nds, file: String) {
        let localized = localize_pathbuf(file).to_string_lossy().into_owned();

        let mut raw: OsString = localized.clone().into();
        raw.push(".context");
        let context_path = PathBuf::from(raw).to_string_lossy().into_owned();

        let context = SavestateContext {
            timestamp: self.time,
            replay: self.replay.as_ref().map(|replay| SavestateContextReplay {
                name: replay.0.name.clone(),
                inputs: replay.0.inputs.clone(),
            }),
        };

        let context_str = serde_yaml::to_string(&context).unwrap();
        std::fs::write(context_path, context_str)
            .expect("Couldn't write savestate context object to file");

        assert!(nds.write_savestate(localized));
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

    // NOTE: rodio requires the OutputStream not to go out of scope until the end
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let audio = Sink::try_new(&stream_handle).unwrap();

    let emu = Arc::new(Mutex::new(Emu::new(start_time, audio, replay)));

    let game_emu = emu.clone();
    let mut game_thread = Some(tokio::spawn(async move {
        game(game_emu, cart, save).await;
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
                    game_thread.take();
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
                        EmuAction::ToggleReplayMode => {
                            if let ElementState::Released = state {
                                return;
                            }
                            if let Some(state) = emu.lock().unwrap().replay.as_mut() {
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
                        EmuAction::SaveReplay => {
                            if let ElementState::Released = state {
                                return;
                            }
                            emu.lock().unwrap().replay_save_request = true;
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

async fn game(emu: Arc<Mutex<Emu>>, cart: Vec<u8>, save: Option<Vec<u8>>) {
    let mut ds_lock = melon::nds::INSTANCE.lock().await;
    let mut ds = ds_lock.take().unwrap();

    ds.load_cart(&cart, save.as_deref());

    println!("Needs direct boot? {:?}", ds.needs_direct_boot());

    if ds.needs_direct_boot() {
        ds.setup_direct_boot(String::from("Ultra.nds"));
    }

    ds.start();

    let audio_queue: Arc<Mutex<Vec<i16>>> = Default::default();
    {
        let emu_audio = &mut emu.lock().unwrap().audio;
        emu_audio.append(NdsAudio::new(audio_queue.clone()));
    }

    let mut timer = ttime::interval_at(
        ttime::Instant::now(),
        ttime::Duration::from_nanos(16_666_667),
    );
    timer.set_missed_tick_behavior(ttime::MissedTickBehavior::Skip);
    loop {
        timer.tick().await;

        let mut force_pause = false;
        let emu_state = emu.lock().unwrap().state;

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
                                } else {
                                    println!(
                                        "WARNING: the replay is in recording mode, but \
                                        cannot record new inputs, because the current \
                                        frame extends beyond the last recorded frame"
                                    )
                                }
                                emu_inputs
                            }
                        },
                    }
                };

                // updates static variable for emulator function impl
                // what's a better strategy? maybe the subscriptions?
                *(GAME_TIME.lock().unwrap()) = emu.lock().unwrap().time;

                ds.set_key_mask(nds_key);
                ds.run_frame();

                check_memory(ds.main_ram());

                // audio is sent to the audio queue in 2 lines!
                let output = ds.read_audio_output();
                audio_queue.lock().unwrap().extend(output);

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

                if force_pause || emu_state == EmuState::Step {
                    emu.lock().unwrap().state = EmuState::Pause;
                }
            }
            EmuState::Pause => {}
        }

        emu.lock()
            .map(|mut emu| {
                if let Some(read_path) = emu.savestate_read_request.take() {
                    emu.read_savestate(&mut ds, read_path);
                }
            })
            .unwrap();

        emu.lock()
            .map(|mut emu| {
                if let Some(write_path) = emu.savestate_write_request.take() {
                    emu.write_savestate(&mut ds, write_path);
                }
            })
            .unwrap();

        emu.lock()
            .map(|mut emu| {
                if let Some(write_path) = emu.ram_write_request.take() {
                    let ram = ds.main_ram();
                    std::fs::write(write_path, ram).unwrap();
                    println!("main RAM written to ram.bin");
                }
            })
            .unwrap();

        emu.lock()
            .map(|mut emu| {
                if emu.replay_save_request {
                    emu.replay_save_request = false;

                    if let Some(replay) = &emu.replay {
                        let file = replay.0.name.clone();
                        std::fs::write(file, serde_yaml::to_string(&replay.0).unwrap()).unwrap();
                        println!("saved replay to {}", replay.0.name.to_string_lossy());
                    }
                }
            })
            .unwrap();
    }

    ds.stop();
}

fn check_memory(ram: &[u8]) {
    use std::io::{Seek, SeekFrom};
    let mut mem_cursor = MemCursor::new(ram, MAIN_RAM_OFFSET as u64);
    let actors = ActorCollection::read(&mut mem_cursor).unwrap();
    // jp version stuff
    // mem_cursor
    //     .seek(SeekFrom::Start(0x02049e0c_u64))
    //     .unwrap();
    // // let actors = ActorCollection::read(&mut mem_cursor).unwrap();
    // let actor = Actor::read(&mut mem_cursor).unwrap();
    println!("{:#X?}", actors);
}
