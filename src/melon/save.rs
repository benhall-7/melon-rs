use std::{sync::Mutex, path::PathBuf};

use once_cell::sync::Lazy;

pub static SAVE_BUFFER: Lazy<Mutex<Vec<u8>>> = Lazy::new(|| Mutex::new(vec![]));

pub fn write_save(save_data: &[u8], write_offset: usize, write_len: usize) {
    let mut save_buffer = SAVE_BUFFER.lock().unwrap();
    if save_data.len() != save_buffer.len() {
        *save_buffer = save_data.to_owned();
    } else if write_offset + write_len <= save_data.len() {
        save_buffer[write_offset..][..write_len]
            .clone_from_slice(&save_data[write_offset..][..write_len]);
    } else {
        save_buffer[write_offset..].clone_from_slice(&save_data[write_offset..]);

        let overflow_len = (write_offset + write_len - save_data.len()).min(save_data.len());
        save_buffer[..overflow_len].clone_from_slice(&save_data[..overflow_len]);
    }
}

pub fn update_save(path: PathBuf) {
    let save_contents = SAVE_BUFFER.lock().unwrap().clone();
    std::fs::write(path, save_contents).unwrap();
}
