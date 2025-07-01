use tanitc_ast::Ast;
use tanitc_messages::{location::Location, Errors, Message, Warnings};

pub mod grammar;

pub struct Parser {
    errors: Errors,
    warnings: Warnings,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            errors: Errors::new(),
            warnings: Warnings::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Ast, Message> {
        Err(Message::new(Location::default(), "hello"))
    }

    pub fn error(&mut self, mut err: Message) {
        err.text = format!("Syntax error: {}", err.text);
        self.errors.push(err);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Syntax warning: {}", warn.text);
        self.errors.push(warn);
    }

    pub fn get_errors(&mut self) -> Errors {
        std::mem::take(&mut self.errors)
    }

    pub fn get_warnings(&mut self) -> Warnings {
        std::mem::take(&mut self.warnings)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
