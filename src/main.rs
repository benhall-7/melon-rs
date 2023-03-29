use std::borrow::Cow;

use glium::{
    glutin, implement_vertex,
    texture::{self, ClientFormat},
    uniform, Surface,
};

use crate::melon::{init_renderer, set_render_settings};

pub mod config;
pub mod events;
pub mod melon;

fn main() {
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

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
    }

    implement_vertex!(Vertex, position, tex_coords);

    // ripped straight from tutorial until I can get something rendering in the first place
    let vertex1 = Vertex {
        position: [-1.0, 0.0],
        tex_coords: [0.0, 0.0],
    };
    let vertex2 = Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 1.0],
    };
    let vertex3 = Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 1.0],
    };
    let vertex4 = Vertex {
        position: [1.0, 0.0],
        tex_coords: [1.0, 0.0],
    };
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

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

    events_loop.run(move |ev, _, control_flow| {
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);

        ds.run_frame();
        ds.update_framebuffers();

        let mut frame = display.draw();

        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        let top_screen = texture::RawImage2d {
            data: Cow::Borrowed(&ds.top_frame),
            width: 256,
            height: 192,
            format: ClientFormat::U8U8U8U8,
        };
        // let bottom_screen =
        //     texture::RawImage2d::from_raw_rgb_reversed(&ds.bottom_frame, (256, 192));
        let top_tex = texture::SrgbTexture2d::new(&display, top_screen).unwrap();
        // let bottom_tex = texture::SrgbTexture2d::new(&display, bottom_screen).unwrap();

        let uniforms = uniform! {tex: &top_tex};

        frame
            .draw(
                &vertex_buffer,
                indices,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        frame.finish().unwrap();

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        if let glutin::event::Event::WindowEvent { event, .. } = ev {
            if event == glutin::event::WindowEvent::CloseRequested {
                *control_flow = glutin::event_loop::ControlFlow::Exit;

                ds.stop();
            }
        }
    });
}
