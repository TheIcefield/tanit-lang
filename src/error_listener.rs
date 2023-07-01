use crate::lexer::Location;

pub type Errors = Vec<String>;

#[derive(Default)]
pub struct ErrorListener {
    errors: Errors,
}

impl ErrorListener {
    pub fn new() -> Self {
        Self { errors: Errors::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn syntax_error(&mut self, message: &str, location: Location) {
        self.errors.push(format!("Syntax Error as [{}]: {}", location, message));
    }

    pub fn semantic_error(&mut self, message: &str, location: Location) {
        self.errors.push(format!("Semantic Error as [{}]: {}", location, message));
    }

    pub fn take_errors(&mut self) -> Errors {
        std::mem::take(&mut self.errors)
    }

    pub fn dump_errors(&self) {
        for error in self.errors.iter() {
            println!("{}", error);
        }
    }
}
