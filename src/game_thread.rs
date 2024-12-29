use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};

use crate::melon::kssu::{addresses::MAIN_RAM_OFFSET, io::MemCursor, ActorCollection};
use crate::melon::nds::{input::NdsKeyMask, Nds};

use crate::frontend::{EmuState, Frontend, ReplayState};

pub struct GameThread {
    ds: Nds,
    emu: Arc<Mutex<Frontend>>,
    // audio_queue: Arc<Mutex<Vec<i16>>>,
}

impl GameThread {
    pub fn new(
        emu: Arc<Mutex<Frontend>>,
        // audio: Arc<Mutex<Vec<i16>>>,
        cart: Vec<u8>,
        save: Option<Vec<u8>>,
        time: DateTime<Utc>,
    ) -> Self {
        let mut ds = Nds::new();

        ds.set_nds_cart(&cart, save.as_deref());
        ds.set_time(time);

        println!("Needs direct boot? {:?}", ds.needs_direct_boot());

        if ds.needs_direct_boot() {
            ds.setup_direct_boot(String::from("Ultra.nds"));
        }

        ds.start();

        GameThread {
            ds,
            emu,
            // audio_queue: audio,
        }
    }

    pub fn execute(&mut self) {
        // check emu state and
        let emu_state = self.emu.lock().unwrap().state;

        match emu_state {
            EmuState::Stop => return,
            EmuState::Run | EmuState::Step => {
                let mut force_pause = false;

                self.run_frame(&mut force_pause);

                if force_pause || emu_state == EmuState::Step {
                    self.emu.lock().unwrap().state = EmuState::Pause;
                }
            }
            EmuState::Pause => {}
        }

        self.emu
            .lock()
            .map(|mut emu| {
                if let Some(read_path) = emu.requests.savestate_read_request.take() {
                    emu.read_savestate(&mut self.ds, read_path);
                }
            })
            .unwrap();

        self.emu
            .lock()
            .map(|mut emu| {
                if let Some(write_path) = emu.requests.savestate_write_request.take() {
                    emu.write_savestate(&mut self.ds, write_path);
                }
            })
            .unwrap();

        self.emu
            .lock()
            .map(|mut emu| {
                if let Some(write_path) = emu.requests.ram_write_request.take() {
                    let ram = self.ds.main_ram();
                    std::fs::write(write_path, ram).unwrap();
                    println!("main RAM written to ram.bin");
                }
            })
            .unwrap();

        self.emu
            .lock()
            .map(|mut emu| {
                if emu.requests.replay_save_request {
                    emu.requests.replay_save_request = false;

                    if let Some(replay) = &emu.replay {
                        let file = replay.0.name.clone();
                        std::fs::write(file, serde_yaml::to_string(&replay.0).unwrap()).unwrap();
                        println!("saved replay to {}", replay.0.name.to_string_lossy());
                    }
                }
            })
            .unwrap();
    }

    // TODO: figure out a better place for the force_pause logic
    pub fn run_frame(&mut self, force_pause: &mut bool) {
        let nds_key = {
            let mut lock = self.emu.lock().unwrap();
            let emu_inputs = lock.nds_input;
            match lock.replay.as_mut() {
                None => emu_inputs,
                Some((replay, replay_state)) => match *replay_state {
                    ReplayState::Playing => {
                        let current_frame = self.ds.current_frame() as usize;
                        if current_frame < replay.inputs.len() {
                            if current_frame == replay.inputs.len() - 1 {
                                *force_pause = true;
                            }
                            NdsKeyMask::from_bits_retain(replay.inputs[current_frame])
                        } else {
                            emu_inputs
                        }
                    }
                    ReplayState::Recording => {
                        let current_frame = self.ds.current_frame() as usize;
                        if current_frame <= replay.inputs.len() {
                            replay.inputs.splice(current_frame.., [emu_inputs.bits()]);
                        } else {
                            println!(
                                "WARNING: the replay is in recording mode, but \
                                cannot record new inputs, because the current \
                                frame extends beyond the last recorded frame"
                            )
                        }
                        emu_inputs
                    }
                },
            }
        };

        self.ds.set_key_mask(nds_key);
        self.ds.run_frame();

        check_memory(self.ds.main_ram());

        let output = self.ds.read_audio_output();
        // TODO: how to shorten nested arcs?
        self.emu
            .lock()
            .unwrap()
            .audio
            .lock()
            .unwrap()
            .extend(output);

        println!("Frame {}", self.ds.current_frame());

        self.emu
            .lock()
            .map(|mut mutex| {
                self.ds.update_framebuffers(&mut mutex.top_frame, false);
                self.ds.update_framebuffers(&mut mutex.bottom_frame, true);
            })
            .unwrap();
    }
}

// TODO: game interface should be in its own file for sure
fn check_memory(ram: &[u8]) {
    // use std::io::{Seek, SeekFrom};
    let mut mem_cursor = MemCursor::new(ram, MAIN_RAM_OFFSET as u64);
    let actors = ActorCollection::read(&mut mem_cursor).unwrap();
    // jp version stuff
    // mem_cursor
    //     .seek(SeekFrom::Start(0x02049e0c_u64))
    //     .unwrap();
    // // let actors = ActorCollection::read(&mut mem_cursor).unwrap();
    // let actor = Actor::read(&mut mem_cursor).unwrap();
    println!("{:#?}", actors);
}
