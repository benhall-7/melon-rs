use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::events::Event;

pub static STOP_EMU: Lazy<Mutex<Event<()>>> = Lazy::new(|| Mutex::new(Event::default()));
pub static LAN_DEINIT: Lazy<Mutex<Event<()>>> = Lazy::new(|| Mutex::new(Event::default()));
// pub static 
