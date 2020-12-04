use chandler::ui::*;

pub struct NullUiHandler {
    cancel_check: Box<dyn Fn() -> bool>,
}

impl NullUiHandler {
    #[allow(dead_code)]
    pub fn new(cancel_check: Box<dyn Fn() -> bool>) -> Self {
        Self { cancel_check }
    }
}

impl ChandlerUiHandler for NullUiHandler {
    fn event(&mut self, _: &UiEvent) {}
    fn is_cancelled(&self) -> bool {
        (self.cancel_check)()
    }
}
