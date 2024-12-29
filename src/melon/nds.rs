use std::io::Write;

use chrono::{DateTime, Datelike, Timelike, Utc};
use cxx::{CxxVector, UniquePtr};

use super::sys;

pub mod input;

pub struct Nds(UniquePtr<sys::NDS>);

impl Nds {
    pub const AUDIO_CHANNELS: u16 = 2;
    pub const AUDIO_SAMPLE_RATE: u32 = 32824;

    pub fn new() -> Self {
        let mut nds = Nds(sys::New_NDS());

        // nds.init_renderer();
        // nds.set_render_settings();
        nds.reset();

        nds
    }

    // fn set_console_type(&mut self, console: ConsoleType) {
    //     let val = console as i32;
    //     sys::nds::SetConsoleType(val);
    // }

    pub fn cart_inserted(&self) -> bool {
        self.0.CartInserted()
    }

    pub fn set_key_mask(&mut self, key_mask: input::NdsKeyMask) {
        self.0.pin_mut().SetKeyMask(!key_mask.bits())
    }

    // pub fn is_lid_closed(&self) -> bool {
    //     sys::nds::IsLidClosed()
    // }

    // pub fn set_lid_closed(&mut self, closed: bool) {
    //     sys::nds::SetLidClosed(closed);
    // }

    pub fn set_nds_cart(&mut self, rom: &[u8], save: Option<&[u8]>) {
        unsafe {
            let cart = sys::ParseROMWithSave(
                rom.as_ptr(),
                rom.len() as u32,
                save.map(|data| data.as_ptr())
                    .unwrap_or_else(std::ptr::null::<u8>),
                save.map(|data| data.len() as u32).unwrap_or_default(),
            );
            sys::NDS_SetNDSCart(self.0.pin_mut(), cart);
        }
    }

    pub fn set_time(&mut self, time: DateTime<Utc>) {
        sys::RTC_SetDateTime(
            self.0.pin_mut(),
            time.year() as i32,
            time.month() as i32,
            time.day() as i32,
            time.hour() as i32,
            time.minute() as i32,
            time.second() as i32,
        )
    }

    pub fn needs_direct_boot(&self) -> bool {
        self.0.NeedsDirectBoot()
    }

    pub fn setup_direct_boot(&mut self, rom_name: String) {
        unsafe {
            sys::NDS_SetupDirectBoot(self.0.pin_mut(), rom_name);
        }
    }

    pub fn start(&mut self) {
        self.0.pin_mut().Start();
    }

    // pub fn stop(&mut self) {
    //     sys::nds::Stop();
    // }

    pub fn reset(&mut self) {
        self.0.pin_mut().Reset();
    }

    /// Emulates a frame. Returns number of scanlines from GPU module
    pub fn run_frame(&mut self) -> u32 {
        self.0.pin_mut().RunFrame()
    }

    pub fn update_framebuffers(&self, dest: &mut [u8], bottom: bool) -> bool {
        assert_eq!(dest.len(), 256 * 192 * 4);
        unsafe {
            sys::Copy_Framebuffers(
                self.0.as_ref().expect("Couldn't get ref to pin"),
                dest.as_mut_ptr(),
                bottom,
            )
        }
    }

    pub fn read_audio_output(&mut self) -> Vec<i16> {
        let mut buffer = [0i16; 1024 * 2];
        let samples_read =
            unsafe { sys::SPU_ReadOutput(self.0.pin_mut(), &mut buffer as *mut i16, 1024) };
        buffer[0..2 * samples_read as usize].into()
    }

    pub fn read_savestate(&mut self, file: String) -> bool {
        let contents = std::fs::read(file).expect("Couldn't open savestate file");
        unsafe { sys::ReadSavestate(self.0.pin_mut(), contents.as_ptr(), contents.len() as i32) }
    }

    pub fn write_savestate(&mut self, file: String) -> bool {
        let mut handle = std::fs::File::create(file).expect("Couldn't create/open savestate file");
        let data: UniquePtr<CxxVector<u8>> = CxxVector::new();
        unsafe {
            let result = sys::WriteSavestate(self.0.pin_mut());
            if result.len() > 0 {
                handle
                    .write_all((*data).as_slice())
                    .expect("Couldn't write contents of savestate");
                true
            } else {
                false
            }
        }
    }

    pub fn current_frame(&self) -> u32 {
        unsafe { sys::CurrentFrame(self.0.as_ref().expect("Couldn't get ref to pin")) }
    }

    pub fn main_ram(&self) -> &[u8] {
        unsafe {
            let max_size = sys::MainRAMMaxSize(&self.0) as usize;
            std::slice::from_raw_parts(
                sys::MainRAM(self.0.as_ref().expect("Couldn't get ref to pin")),
                max_size,
            )
        }
    }

    pub fn main_ram_mut(&mut self) -> &mut [u8] {
        unsafe {
            let max_size = sys::MainRAMMaxSize(&self.0) as usize;
            std::slice::from_raw_parts_mut(sys::MainRAMMut(self.0.pin_mut()), max_size)
        }
    }

    // fn init_renderer(&mut self) {
    //     sys::gpu::InitRenderer(0);
    // }
}

unsafe impl Send for Nds {}
