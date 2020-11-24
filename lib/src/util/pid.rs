use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use log::debug;
use sysinfo::Pid;

pub struct PidLock {
    path: PathBuf,
}

impl PidLock {
    pub fn acquire(path: impl AsRef<Path>) -> Option<Self> {
        let path = env::current_dir().unwrap().join(path);
        debug!("Trying to acquire PID lock at {:?}", path);

        if path.exists() {
            debug!("PID file found at {:?}", path);
            if let Ok(mut file) = fs::File::open(&path) {
                let mut pid = String::new();
                file.read_to_string(&mut pid).unwrap();

                let pid: Pid = pid.parse().unwrap();

                debug!("File contains PID {}.", pid);
                if process_exists(pid) {
                    // Process already exists, cannot get lock.
                    debug!("Process with PID {} exists, cannot get lock.", pid);
                    return None;
                }
            }
        }

        // Try to create a PID file...
        if let Ok(mut file) = fs::File::create(&path) {
            // Write our PID to the newly created file
            file.write(format!("{}", std::process::id()).as_bytes()).unwrap();
        } else {
            return None;
        };

        Some(Self {
            path: path.to_path_buf(),
        })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        debug!("Dropping PID-lock at {}", self.path.display());
        fs::remove_file(&self.path).unwrap();
    }
}

fn process_exists(pid: Pid) -> bool {
    use sysinfo::{System, SystemExt};

    let mut sys = System::new_all();

    if let Some(p) = sys.get_process(pid) {
        true
    } else {
        false
    }
}
