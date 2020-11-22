use chandler::ui::*;

pub struct NullUiHandler;

impl NullUiHandler {
    pub fn new() -> Self {
        Self
    }
}

impl ChandlerUiHandler for NullUiHandler {
    fn event(&mut self, _: &UiEvent) {}
}
