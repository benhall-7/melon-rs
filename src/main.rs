use crate::melon::init_renderer;

pub mod config;
pub mod events;
pub mod melon;

fn main() {
    // 1. The **winit::EventsLoop** for handling events.
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    // 2. Parameters for building the Window.
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Hello world");
    // 3. Parameters for building the OpenGL context.
    let cb = glium::glutin::ContextBuilder::new();
    // 4. Build the Display with the given window and OpenGL context parameters and register the
    //    window with the events_loop.
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut lock = melon::nds::INSTANCE.lock().unwrap();
    let mut ds = lock.take().unwrap();

    ds.load_cart(
        &std::fs::read("/Users/benjamin/Desktop/ds/Ultra.nds").unwrap(),
        None,
    );

    init_renderer();

    println!("{}", ds.cart_inserted());

    let frame = display.draw();

    ds.start();

    ds.run_frame();

    ds.stop();

    frame.finish().unwrap();
}
