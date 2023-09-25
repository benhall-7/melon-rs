use std::path::PathBuf;

pub fn localize_pathbuf(path: String) -> PathBuf {
    let pathbuf = PathBuf::from(path);
    if pathbuf.is_absolute() {
        pathbuf
    } else {
        std::env::current_exe()
            .expect("Couldn't get target executable path")
            .parent()
            .expect("Failed to get path to current executable's parent folder")
            .join("melon")
            .join(pathbuf)
    }
}
