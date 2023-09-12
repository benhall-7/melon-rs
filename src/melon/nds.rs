use std::{ffi::OsString, path::PathBuf, sync::Mutex};

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

use super::sys::{self, glue::localize_pathbuf};

pub mod input;

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

            nds.init_renderer();
            nds.set_render_settings();
            nds.reset();

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

    pub fn set_key_mask(&mut self, key_mask: input::NdsKeyMask) {
        sys::nds::SetKeyMask(!key_mask.bits())
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

    pub fn needs_direct_boot(&self) -> bool {
        sys::nds::NeedsDirectBoot()
    }

    pub fn setup_direct_boot(&mut self, rom_name: String) {
        sys::platform::glue::NDS_SetupDirectBoot(rom_name);
    }

    pub fn start(&mut self) {
        sys::nds::Start();
    }

    pub fn stop(&mut self) {
        sys::nds::Stop();
    }

    pub fn reset(&mut self) {
        sys::nds::Reset();
    }

    // Emulates a frame. Returns number of scanlines from GPU module
    pub fn run_frame(&mut self) -> u32 {
        sys::nds::RunFrame()
    }

    pub fn update_framebuffers(&self, dest: &mut [u8], bottom: bool) -> bool {
        assert_eq!(dest.len(), 256 * 192 * 4);
        unsafe { sys::platform::glue::Copy_Framebuffers(dest.as_mut_ptr(), bottom) }
    }

    pub fn set_render_settings(&mut self) {
        sys::gpu::SetRenderSettings(
            0,
            &mut sys::gpu::RenderSettings {
                Soft_Threaded: false,
                GL_ScaleFactor: 1,
                GL_BetterPolygons: false,
            },
        );
    }

    pub fn read_savestate(&mut self, file: String) -> (bool, DateTime<Utc>) {
        let localized = localize_pathbuf(file).to_string_lossy().into_owned();

        let mut raw: OsString = localized.clone().into();
        raw.push(".timestamp");
        let timestamp_path = PathBuf::from(raw).to_string_lossy().into_owned();

        let timestamp_str = std::fs::read_to_string(&timestamp_path)
            .unwrap_or_else(|_| panic!("Couldn't read timestamp file: {}", timestamp_path));
        let timestamp: DateTime<Utc> = DateTime::parse_from_rfc2822(&timestamp_str)
            .expect("Couldn't parse timestamp file: {}")
            .into();

        (sys::platform::glue::ReadSavestate(localized), timestamp)
    }

    pub fn write_savestate(&mut self, file: String, timestamp: DateTime<Utc>) -> bool {
        let localized = localize_pathbuf(file).to_string_lossy().into_owned();

        let mut raw: OsString = localized.clone().into();
        raw.push(".timestamp");
        let timestamp_path = PathBuf::from(raw).to_string_lossy().into_owned();

        let timestamp_str = timestamp.to_rfc2822();
        std::fs::write(&timestamp_path, timestamp_str)
            .unwrap_or_else(|_| panic!("Couldn't write timestamp file: {}", timestamp_path));

        sys::platform::glue::WriteSavestate(localized)
    }

    pub fn current_frame(&self) -> u32 {
        sys::platform::glue::CurrentFrame()
    }

    pub fn main_ram(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                sys::platform::glue::MainRAM(),
                sys::platform::glue::MainRAMMaxSize() as usize,
            )
        }
    }

    pub fn main_ram_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                sys::platform::glue::MainRAM(),
                sys::platform::glue::MainRAMMaxSize() as usize,
            )
        }
    }

    fn init_renderer(&mut self) {
        sys::gpu::InitRenderer(0);
    }
}

impl Drop for NDS {
    fn drop(&mut self) {
        sys::nds::DeInit()
    }
}
