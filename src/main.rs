pub mod melon;
pub mod events;
pub mod config;

fn main() {
    // use glium::glutin;

    // let mut event_loop = glutin::event_loop::EventLoop::new();
    // let wb = glutin::window::WindowBuilder::new();
    // let cb = glutin::ContextBuilder::new();
    // let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut lock = melon::nds::INSTANCE.lock().unwrap();
    let ds = lock.take().unwrap();

    println!("{}", ds.cart_inserted());
}
