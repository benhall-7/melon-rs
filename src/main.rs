use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::spawn,
};

use config::{Config, ConfigFile, EmuAction, EmuInput};
use glium::glutin::{
    self,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
};
use melon::nds::input::NdsKey;
use window::{draw, get_draw_data};
use winit::event::ModifiersState;

use crate::melon::{nds::input::NdsKeyMask, save, sys::glue::localize_path};

pub mod config;
pub mod events;
pub mod melon;
pub mod window;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Run,
    Pause,
    Stop,
    Step,
}

#[derive(Debug)]
pub struct Emu {
    pub top_frame: [u8; 256 * 192 * 4],
    pub bottom_frame: [u8; 256 * 192 * 4],
    pub nds_input: NdsKeyMask,
    pub key_modifiers: ModifiersState,
    pub state: EmuState,
    pub savestate_write_request: Option<String>,
    pub savestate_read_request: Option<String>,
}

impl Emu {
    pub fn new() -> Self {
        Emu {
            top_frame: [0; 256 * 192 * 4],
            bottom_frame: [0; 256 * 192 * 4],
            nds_input: NdsKeyMask::empty(),
            key_modifiers: ModifiersState::empty(),
            state: EmuState::Run,
            savestate_read_request: None,
            savestate_write_request: None,
        }
    }
}

impl Default for Emu {
    fn default() -> Self {
        Emu::new()
    }
}

fn main() {
    let config: Config = std::fs::read_to_string("config.yml")
        .ok()
        .and_then(|yml| serde_yaml::from_str::<ConfigFile>(&yml).ok())
        .map(Into::into)
        .unwrap_or_default();

    let emu = Arc::new(Mutex::new(Emu::new()));

    let game_emu = emu.clone();
    let mut game_thread = Some(spawn(|| game(game_emu)));

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
                    }
                }
                _ => {}
            }
        }
    });
}

fn game(emu: Arc<Mutex<Emu>>) {
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
        let emu_state = emu.lock().unwrap().state;

        match emu_state {
            EmuState::Stop => break,
            EmuState::Run | EmuState::Step => {
                ds.set_key_mask(emu.lock().unwrap().nds_input);
                ds.run_frame();

                emu.lock()
                    .map(|mut mutex| {
                        ds.update_framebuffers(&mut mutex.top_frame, false);
                        ds.update_framebuffers(&mut mutex.bottom_frame, true);
                    })
                    .unwrap();

                if let EmuState::Step = emu_state {
                    emu.lock().unwrap().state = EmuState::Pause;
                }
            }
            EmuState::Pause => {}
        }

        let (savestate_read_request, savestate_write_request) = emu
            .lock()
            .map(|mut lock| {
                (
                    lock.savestate_read_request.take(),
                    lock.savestate_write_request.take(),
                )
            })
            .unwrap();

        savestate_read_request
            .map(localize_path)
            .map(|read_path| ds.read_savestate(read_path));
        savestate_write_request
            .map(localize_path)
            .map(|write_path| ds.write_savestate(write_path));

        fps.tick();
    }

    ds.stop();
}
