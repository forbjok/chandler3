use chandler::ui::*;

pub struct NullUiHandler;

impl NullUiHandler {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl ChandlerUiHandler for NullUiHandler {
    fn event(&mut self, _: &UiEvent) {}
    fn is_cancelled(&self) -> bool {
        false
    }
}
