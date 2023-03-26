use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::sys;

pub static INSTANCE: Lazy<Mutex<Option<NDS>>> =
    Lazy::new(|| Mutex::new(Some(NDS::new().expect("Couldn't initialize NDS"))));

pub struct NDS {
    pub top_frame: [u32; 256 * 192],
    pub bottom_frame: [u32; 256 * 192],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleType {
    DS = 0,
    DSi = 1,
}

impl NDS {
    fn new() -> Result<Self, ()> {
        let res = sys::nds::Init();
        if res {
            let mut nds = NDS {
                top_frame: [0; 256 * 192],
                bottom_frame: [0; 256 * 192],
            };
            nds.set_console_type(ConsoleType::DS);
            // nds.reset();
            Ok(nds)
        } else {
            Err(())
        }
    }

    fn set_console_type(&mut self, console: ConsoleType) {
        let val = console as i32;
        sys::nds::SetConsoleType(val);
    }

    pub fn reset(&mut self) {
        sys::nds::Reset();
        self.top_frame.fill(0);
        self.bottom_frame.fill(0);
    }

    pub fn cart_inserted(&self) -> bool {
        sys::nds::CartInserted()
    }

    pub fn is_lid_closed(&self) -> bool {
        sys::nds::IsLidClosed()
    }

    pub fn set_lid_closed(&mut self, closed: bool) {
        sys::nds::SetLidClosed(closed);
    }

    pub fn load_cart(&mut self, rom: &[u8], save: Option<&[u8]>) -> bool {
        unsafe {
            sys::nds::LoadCart(
                rom.as_ptr(),
                rom.len() as u32,
                save.map(|data| data.as_ptr())
                    .unwrap_or_else(std::ptr::null::<u8>),
                save.map(|data| data.len() as u32).unwrap_or_default(),
            )
        }
    }

    pub fn start(&mut self) {
        sys::nds::Start();
    }

    pub fn stop(&mut self) {
        sys::nds::Stop();
        self.top_frame.fill(0);
        self.bottom_frame.fill(0);
    }

    // Emulates a frame. Returns number of scanlines from GPU module
    pub fn run_frame(&mut self) -> u32 {
        sys::nds::RunFrame()
    }

    pub fn update_framebuffers(&mut self) -> bool {
        unsafe {
            sys::platform::glue::Copy_Framebuffers(
                self.top_frame.as_mut_ptr(),
                self.bottom_frame.as_mut_ptr(),
            )
        }
    }
}

impl Drop for NDS {
    fn drop(&mut self) {
        sys::nds::DeInit()
    }
}
