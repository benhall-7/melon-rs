use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::sys;

pub static INSTANCE: Lazy<Mutex<Option<NDS>>> =
    Lazy::new(|| Mutex::new(Some(NDS::new().expect("Couldn't initialize NDS"))));

pub struct NDS(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleType {
    DS = 0,
    DSi = 1,
}

impl NDS {
    fn new() -> Result<Self, ()> {
        let res = sys::nds::Init();
        if res {
            let mut nds = NDS(());
            nds.set_console_type(ConsoleType::DS);
            Ok(nds)
        } else {
            Err(())
        }
    }

    fn set_console_type(&mut self, console: ConsoleType) {
        let val = console as i32;
        sys::nds::SetConsoleType(val);
    }

    pub fn cart_inserted(&self) -> bool {
        sys::nds::CartInserted()
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
}

impl Drop for NDS {
    fn drop(&mut self) {
        sys::nds::DeInit()
    }
}
