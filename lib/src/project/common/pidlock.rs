use std::path::Path;

use crate::util::pid::PidLock;

pub fn acquire_pidlock(root_path: &Path, filename: &str) -> Option<PidLock> {
    PidLock::acquire(root_path.join(filename))
}
