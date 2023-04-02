use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
    thread::spawn,
};

use glium::{
    glutin::{
        self,
        event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    },
    implement_vertex,
    texture::{self, ClientFormat},
    uniform, Surface,
};

use crate::melon::{init_renderer, nds::input::NdsKey, set_render_settings};

pub mod config;
pub mod events;
pub mod melon;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Run,
    Pause,
    Stop,
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

fn main() {
    let emu = Arc::new(Mutex::new(Emu::new()));

    let game_emu = emu.clone();
    let mut game_thread = Some(spawn(|| game(game_emu)));

    // 1. The **winit::EventsLoop** for handling events.
    let events_loop = glutin::event_loop::EventLoop::new();
    // 2. Parameters for building the Window.
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(256.0, 192.0 * 2.0))
        .with_title("melon-rs");
    // 3. Parameters for building the OpenGL context.
    let cb = glutin::ContextBuilder::new();
    // 4. Build the Display with the given window and OpenGL context parameters and register the
    //    window with the events_loop.
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let vertex_shader_src = include_str!("main.vert");
    let fragment_shader_src = include_str!("main.frag");

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
    }

    implement_vertex!(Vertex, position, tex_coords);

    let vertex_a1 = Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0],
    };
    let vertex_a2 = Vertex {
        position: [-1.0, 0.0],
        tex_coords: [0.0, 1.0],
    };
    let vertex_a3 = Vertex {
        position: [1.0, 0.0],
        tex_coords: [1.0, 1.0],
    };
    let vertex_a4 = Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    };
    let shape_a = vec![vertex_a1, vertex_a2, vertex_a3, vertex_a4];

    let vertex_b1 = Vertex {
        position: [-1.0, 0.0],
        tex_coords: [0.0, 0.0],
    };
    let vertex_b2 = Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    };
    let vertex_b3 = Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    };
    let vertex_b4 = Vertex {
        position: [1.0, 0.0],
        tex_coords: [1.0, 0.0],
    };
    let shape_b = vec![vertex_b1, vertex_b2, vertex_b3, vertex_b4];

    let vertex_buffer_a = glium::VertexBuffer::new(&display, &shape_a).unwrap();
    let vertex_buffer_b = glium::VertexBuffer::new(&display, &shape_b).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

    events_loop.run(move |ev, _target, control_flow| {
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_secs_f64(1.0 / 60.0);

        let (top_frame, bottom_frame) = {
            let emu_lock = (*emu).lock().unwrap();
            (emu_lock.top_frame, emu_lock.bottom_frame)
        };

        let mut frame = display.draw();

        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        let top_screen = texture::RawImage2d {
            data: Cow::Borrowed(&top_frame),
            width: 256,
            height: 192,
            format: ClientFormat::U8U8U8U8,
        };
        let bottom_screen = texture::RawImage2d {
            data: Cow::Borrowed(&bottom_frame),
            width: 256,
            height: 192,
            format: ClientFormat::U8U8U8U8,
        };
        let top_tex = texture::SrgbTexture2d::new(&display, top_screen).unwrap();
        let bottom_tex = texture::SrgbTexture2d::new(&display, bottom_screen).unwrap();

        let uniforms_a = uniform! {tex: &top_tex};
        let uniforms_b = uniform! {tex: &bottom_tex};

        frame
            .draw(
                &vertex_buffer_a,
                indices,
                &program,
                &uniforms_a,
                &Default::default(),
            )
            .unwrap();
        frame
            .draw(
                &vertex_buffer_b,
                indices,
                &program,
                &uniforms_b,
                &Default::default(),
            )
            .unwrap();

        frame.finish().unwrap();

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
                    let nds_key = match input.virtual_keycode.unwrap() {
                        VirtualKeyCode::K => NdsKey::A,
                        VirtualKeyCode::M => NdsKey::B,
                        VirtualKeyCode::J => NdsKey::X,
                        VirtualKeyCode::N => NdsKey::Y,
                        VirtualKeyCode::W => NdsKey::Up,
                        VirtualKeyCode::A => NdsKey::Left,
                        VirtualKeyCode::S => NdsKey::Down,
                        VirtualKeyCode::D => NdsKey::Right,
                        VirtualKeyCode::Q => NdsKey::L,
                        VirtualKeyCode::P => NdsKey::R,
                        VirtualKeyCode::Space => NdsKey::Start,
                        VirtualKeyCode::X => NdsKey::Select,
                        _ => return,
                    };

                    match input.state {
                        ElementState::Pressed => emu.lock().unwrap().nds_input.insert(nds_key),
                        ElementState::Released => emu.lock().unwrap().nds_input.remove(nds_key),
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

    init_renderer();
    set_render_settings();

    ds.reset();

    ds.load_cart(
        &std::fs::read("/Users/benjamin/Desktop/ds/Ultra.nds").unwrap(),
        None,
    );

    println!("Needs direct boot? {:?}", ds.needs_direct_boot());

    if ds.needs_direct_boot() {
        ds.setup_direct_boot(String::from("Ultra.nds"));
    }

    ds.start();

    let mut fps = fps_clock::FpsClock::new(60);
    loop {
        ds.set_key_mask(emu.lock().unwrap().nds_input);
        ds.run_frame();

        emu.lock()
            .map(|mut mutex| {
                ds.update_framebuffers(&mut mutex.top_frame, false);
                ds.update_framebuffers(&mut mutex.bottom_frame, true);
            })
            .unwrap();

        if let EmuState::Stop = emu.lock().unwrap().state {
            break;
        }

        fps.tick();
    }

    ds.stop();
}
