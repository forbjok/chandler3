use chandler::progress::{ChandlerProgressCallbackHandler, ProgressEvent};

pub struct NullProgressHandler;

impl NullProgressHandler {
    pub fn new() -> Self {
        Self
    }
}

impl ChandlerProgressCallbackHandler for NullProgressHandler {
    fn progress(&mut self, _: &ProgressEvent) {}
}
