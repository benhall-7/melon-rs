use std::sync::{Arc, Mutex};

use clap::Parser;
use glium::glutin::{self, event::Event};
use tokio::time as ttime;

use crate::audio::Audio;
use crate::config::{Config, ConfigFile, StartParams};
use crate::frontend::Frontend;
use crate::game_thread::GameThread;
use crate::window::{draw, get_draw_data};

pub mod args;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod game_thread;
pub mod melon;
pub mod replay;
pub mod utils;
pub mod window;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();

    let config: Config = std::fs::read_to_string("config.yml")
        .ok()
        .map(|yml| serde_yaml::from_str::<ConfigFile>(&yml).unwrap())
        .map(Into::into)
        .unwrap_or_default();

    let StartParams {
        replay,
        game_name,
        save_name,
        start_time,
    } = config.get_start_params(args);

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
    let mut audio = Audio::new();

    let frontend = Frontend::new(audio.get_game_stream(), config.key_map, replay);
    let emu = Arc::new(Mutex::new(frontend));

    let mut game_thread = GameThread::new(emu.clone(), cart, save, start_time);
    let mut game_thread_handle = Some(tokio::spawn(async move {
        let mut timer = ttime::interval_at(
            ttime::Instant::now(),
            ttime::Duration::from_nanos(16_666_667),
        );
        timer.set_missed_tick_behavior(ttime::MissedTickBehavior::Skip);
        loop {
            timer.tick().await;

            game_thread.execute()
        }
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
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        let (top_frame, bottom_frame) = {
            let emu_lock = (*emu).lock().unwrap();
            (emu_lock.top_frame, emu_lock.bottom_frame)
        };

        draw(&display, &draw_data, &top_frame, &bottom_frame);

        if let Event::WindowEvent { event, .. } = ev {
            let escape = emu.lock().unwrap().window_event(event);
            if escape {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
                game_thread_handle.take();
            }
        }
    });
}
