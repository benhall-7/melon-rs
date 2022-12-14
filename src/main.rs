mod melon;

fn main() {
    // use glium::glutin;

    // let mut event_loop = glutin::event_loop::EventLoop::new();
    // let wb = glutin::window::WindowBuilder::new();
    // let cb = glutin::ContextBuilder::new();
    // let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    println!("{}", melon::nds::cart_inserted())
}
