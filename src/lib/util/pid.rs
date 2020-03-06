use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use log::debug;

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

                let pid: u32 = pid.parse().unwrap();

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
        dbg!(&self.path);
        fs::remove_file(&self.path).unwrap();
    }
}

#[cfg(windows)]
fn process_exists(pid: u32) -> bool {
    use winapi::{
        shared::minwindef::{
            DWORD,
            FALSE,
            LPDWORD,
        },
        um::{
            handleapi::CloseHandle,
            minwinbase::STILL_ACTIVE,
            processthreadsapi::{
                GetExitCodeProcess,
                OpenProcess,
            },
            winnt::{
                HANDLE,
                PROCESS_QUERY_INFORMATION,
            }
        }
    };

    // Try to open process
    let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pid as DWORD) };

    if handle.is_null() {
        false
    } else {
        let mut exitcode: LPDWORD = unsafe { std::mem::zeroed() };

        // Try to get exit code for process
        let get_exitcode_result = unsafe { GetExitCodeProcess(handle, exitcode) };

        // Close process handle
        unsafe { CloseHandle(handle) };

        if get_exitcode_result == FALSE {
            true
        } else {
            if unsafe { *exitcode } == STILL_ACTIVE {
                true
            } else {
                false
            }
        }
    }
}

#[cfg(not(windows))]
fn process_exists(pid: u32) -> bool {
    dbg!("PidLock not implemented for this OS!");
    false
}
