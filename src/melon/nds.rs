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
}

impl Drop for NDS {
    fn drop(&mut self) {
        sys::nds::DeInit()
    }
}
