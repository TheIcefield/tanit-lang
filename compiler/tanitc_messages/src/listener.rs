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

    pub fn errors_ref(&self) -> &Errors {
        &self.errors
    }

    pub fn warnings_ref(&self) -> &Warnings {
        &self.warnings
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
