use std::sync::{Arc, Mutex};

use egui::Rect;
use tokio::sync::{mpsc, watch};

use super::InputEvent;

/// Hands host input to the emulator and wakes it between frame ticks.
#[derive(Clone)]
pub struct InputBridge {
    pub tx: mpsc::Sender<InputEvent>,
    wake: watch::Sender<u64>,
    /// Where the bottom screen last landed, for touch mapping between layouts.
    pub bottom_screen: Arc<Mutex<Option<Rect>>>,
}

impl InputBridge {
    pub fn new(tx: mpsc::Sender<InputEvent>) -> (Self, watch::Receiver<u64>) {
        let (wake, wake_rx) = watch::channel(0);
        let bridge = InputBridge {
            tx,
            wake,
            bottom_screen: Arc::new(Mutex::new(None)),
        };
        (bridge, wake_rx)
    }

    pub fn forward(&self, events: impl IntoIterator<Item = InputEvent>) {
        let mut forwarded = false;
        for event in events {
            if let Err(err) = self.tx.try_send(event) {
                println!("WARNING: a host input event was dropped: {err}");
            }
            forwarded = true;
        }
        if forwarded {
            self.wake.send_modify(|n| *n = n.wrapping_add(1));
        }
    }
}
