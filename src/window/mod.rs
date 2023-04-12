use glium::{Display, Program, VertexBuffer};

use std::borrow::Cow;

use glium::{
    implement_vertex,
    texture::{self, ClientFormat},
    uniform, Surface,
};

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

#[derive(Debug)]
pub struct DrawData {
    vertex_buffer_a: VertexBuffer<Vertex>,
    vertex_buffer_b: VertexBuffer<Vertex>,
    program: Program,
}

pub fn get_draw_data(display: &Display) -> DrawData {
    let vertex_shader_src = include_str!("main.vert");
    let fragment_shader_src = include_str!("main.frag");

    let program =
        glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let shape_a = vec![
        Vertex {
            position: [-1.0, 1.0],
            tex_coords: [0.0, 0.0],
        },
        Vertex {
            position: [-1.0, 0.0],
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: [1.0, 0.0],
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 0.0],
        },
    ];

    let shape_b = vec![
        Vertex {
            position: [-1.0, 0.0],
            tex_coords: [0.0, 0.0],
        },
        Vertex {
            position: [-1.0, -1.0],
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: [1.0, 0.0],
            tex_coords: [1.0, 0.0],
        },
    ];

    let vertex_buffer_a = glium::VertexBuffer::new(display, &shape_a).unwrap();
    let vertex_buffer_b = glium::VertexBuffer::new(display, &shape_b).unwrap();

    DrawData {
        vertex_buffer_a,
        vertex_buffer_b,
        program,
    }
}

pub fn draw(display: &Display, draw_data: &DrawData, top_frame: &[u8], bottom_frame: &[u8]) {
    let mut frame = display.draw();

    frame.clear_color(0.0, 0.0, 0.0, 1.0);
    let top_screen = texture::RawImage2d {
        data: Cow::Borrowed(top_frame),
        width: 256,
        height: 192,
        format: ClientFormat::U8U8U8U8,
    };
    let bottom_screen = texture::RawImage2d {
        data: Cow::Borrowed(bottom_frame),
        width: 256,
        height: 192,
        format: ClientFormat::U8U8U8U8,
    };
    let top_tex = texture::SrgbTexture2d::new(display, top_screen).unwrap();
    let bottom_tex = texture::SrgbTexture2d::new(display, bottom_screen).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

    let uniforms_a = uniform! {tex: &top_tex};
    let uniforms_b = uniform! {tex: &bottom_tex};

    frame
        .draw(
            &draw_data.vertex_buffer_a,
            indices,
            &draw_data.program,
            &uniforms_a,
            &Default::default(),
        )
        .unwrap();
    frame
        .draw(
            &draw_data.vertex_buffer_b,
            indices,
            &draw_data.program,
            &uniforms_b,
            &Default::default(),
        )
        .unwrap();

    frame.finish().unwrap();
}
