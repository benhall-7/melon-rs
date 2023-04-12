use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::spawn,
};

use glium::glutin::{
    self,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
};
use window::{draw, get_draw_data};

use crate::melon::{nds::input::NdsKey, save};

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
    pub nds_input: NdsKey,
    pub state: EmuState,
}

impl Emu {
    pub fn new() -> Self {
        Emu {
            top_frame: [0; 256 * 192 * 4],
            bottom_frame: [0; 256 * 192 * 4],
            nds_input: NdsKey::empty(),
            state: EmuState::Run,
        }
    }
}

impl Default for Emu {
    fn default() -> Self {
        Emu::new()
    }
}

#[derive(Debug, PartialEq, Clone)]
enum EmuKey {
    Nds(NdsKey),
    PlayPlause,
    Step,
    Save(PathBuf),
}

fn main() {
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
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode.is_none() {
                        return;
                    }
                    let emu_key = match input.virtual_keycode.unwrap() {
                        VirtualKeyCode::K => EmuKey::Nds(NdsKey::A),
                        VirtualKeyCode::M => EmuKey::Nds(NdsKey::B),
                        VirtualKeyCode::J => EmuKey::Nds(NdsKey::X),
                        VirtualKeyCode::N => EmuKey::Nds(NdsKey::Y),
                        VirtualKeyCode::W => EmuKey::Nds(NdsKey::Up),
                        VirtualKeyCode::A => EmuKey::Nds(NdsKey::Left),
                        VirtualKeyCode::S => EmuKey::Nds(NdsKey::Down),
                        VirtualKeyCode::D => EmuKey::Nds(NdsKey::Right),
                        VirtualKeyCode::Q => EmuKey::Nds(NdsKey::L),
                        VirtualKeyCode::P => EmuKey::Nds(NdsKey::R),
                        VirtualKeyCode::Space => EmuKey::Nds(NdsKey::Start),
                        VirtualKeyCode::X => EmuKey::Nds(NdsKey::Select),
                        VirtualKeyCode::Comma => EmuKey::PlayPlause,
                        VirtualKeyCode::Period => EmuKey::Step,
                        VirtualKeyCode::Z => EmuKey::Save(PathBuf::from("save.bin")),
                        _ => return,
                    };

                    match (emu_key, input.state) {
                        (EmuKey::Nds(nds_key), state) => match state {
                            ElementState::Pressed => emu.lock().unwrap().nds_input.insert(nds_key),
                            ElementState::Released => emu.lock().unwrap().nds_input.remove(nds_key),
                        },
                        (EmuKey::PlayPlause, ElementState::Pressed) => {
                            let emu_state = &mut emu.lock().unwrap().state;
                            match *emu_state {
                                EmuState::Pause | EmuState::Step => *emu_state = EmuState::Run,
                                EmuState::Run => *emu_state = EmuState::Pause,
                                EmuState::Stop => {}
                            }
                        }
                        (EmuKey::PlayPlause, ..) => {}
                        (EmuKey::Step, ElementState::Pressed) => {
                            let emu_state = &mut emu.lock().unwrap().state;
                            match *emu_state {
                                EmuState::Stop => {}
                                _ => *emu_state = EmuState::Step,
                            }
                        }
                        (EmuKey::Step, ..) => {}
                        (EmuKey::Save(path), ElementState::Pressed) => {
                            spawn(|| save::update_save(path));
                        }
                        (EmuKey::Save(_), ..) => {}
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

        fps.tick();
    }

    ds.stop();
}
