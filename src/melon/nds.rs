use std::{io::Write, pin::Pin};

use cxx::UniquePtr;
use tokio::sync::Mutex;

use once_cell::sync::Lazy;

use super::sys;

pub mod audio;
pub mod input;

pub struct Nds(UniquePtr<sys::nds::NDS>);

impl Nds {
    pub fn new() -> Self {
        let mut nds = Nds(sys::shims::New_NDS());

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
            let cart = sys::shims::ParseROMWithSave(
                rom.as_ptr(),
                rom.len() as u32,
                save.map(|data| data.as_ptr())
                    .unwrap_or_else(std::ptr::null::<u8>),
                save.map(|data| data.len() as u32).unwrap_or_default(),
            );
            sys::shims::NDS_SetNDSCart(self.0.pin_mut(), cart);
        }
    }

    pub fn needs_direct_boot(&self) -> bool {
        self.0.NeedsDirectBoot()
    }

    pub fn setup_direct_boot(&mut self, rom_name: String) {
        self.0.pin_mut().SetupDirectBoot();
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

    // Emulates a frame. Returns number of scanlines from GPU module
    pub fn run_frame(&mut self) -> u32 {
        self.0.pin_mut().RunFrame()
    }

    pub fn update_framebuffers(&self, dest: &mut [u8], bottom: bool) -> bool {
        assert_eq!(dest.len(), 256 * 192 * 4);
        unsafe { sys::shims::Copy_Framebuffers(self.0, dest.as_mut_ptr(), bottom) }
    }

    // pub fn set_render_settings(&mut self) {
    //     sys::gpu::SetRenderSettings(
    //         0,
    //         &mut sys::gpu::RenderSettings {
    //             Soft_Threaded: false,
    //             GL_ScaleFactor: 1,
    //             GL_BetterPolygons: false,
    //         },
    //     );
    // }

    pub fn read_audio_output(&mut self) -> Vec<i16> {
        let mut buffer = [0i16; 1024 * 2];
        let samples_read = unsafe { sys::spu::ReadOutput(&mut buffer as *mut i16, 1024) };
        buffer[0..2 * samples_read as usize].into()
    }

    pub fn read_savestate(&mut self, file: String) -> bool {
        let contents = std::fs::read(file).expect("Couldn't open savestate file");
        unsafe { sys::shims::ReadSavestate(self.0.pin_mut(), contents.as_ptr(), contents.len() as i32) }
    }

    pub fn write_savestate(&mut self, file: String) -> bool {
        let handle = std::fs::File::create(file).expect("Couldn't create/open savestate file");
        let store = move |source: *const u8, len: i32| {
            let slice = unsafe { std::slice::from_raw_parts(source, len as usize) };
            handle
                .write_all(slice)
                .expect("Couldn't write file contents for savestate");
        };
        unsafe { sys::shims::WriteSavestate(self.0.pin_mut(), store) }
    }

    pub fn current_frame(&self) -> u32 {
        unsafe { sys::shims::CurrentFrame(self.0) }
    }

    pub fn main_ram(&self) -> &[u8] {
        unsafe {
            let max_size = sys::shims::MainRAMMaxSize(&self.0) as usize;
            std::slice::from_raw_parts(sys::shims::MainRAM(self.0), max_size)
        }
    }

    pub fn main_ram_mut(&mut self) -> &mut [u8] {
        unsafe {
            let max_size = sys::shims::MainRAMMaxSize(&self.0) as usize;
            std::slice::from_raw_parts_mut(sys::shims::MainRAM(self.0.pin_mut()), max_size)
        }
    }

    // fn init_renderer(&mut self) {
    //     sys::gpu::InitRenderer(0);
    // }
}
