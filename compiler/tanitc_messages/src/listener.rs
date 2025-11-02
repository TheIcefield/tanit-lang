use crate::{messages::Message, Errors, Warnings};

#[derive(Default, Clone)]
pub struct MessageListener {
    errors: Errors,
    warnings: Warnings,
}

impl MessageListener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn error(&mut self, msg: Message) {
        self.errors.push(msg);
    }

    pub fn warn(&mut self, msg: Message) {
        self.warnings.push(msg);
    }
}
