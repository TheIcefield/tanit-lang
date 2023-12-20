use crate::{analyzer, lexer::Location};

pub type Errors = Vec<String>;

pub static UNEXPECTED_TOKEN_ERROR_STR: &str = "unexpected token";
pub static UNEXPECTED_END_OF_LINE_ERROR_STR: &str = "end of line wasn\'t expected";
pub static UNEXPECTED_NODE_PARSED_ERROR_STR: &str = "parsed unexpected node";
pub static VARIABLE_DEFINED_WITHOUT_TYPE_ERROR_STR: &str = "variable defined without type";
pub static PARSING_FAILED_ERROR_STR: &str = "some error occured during parsing";
pub static ANALYZING_FAILED_ERROR_STR: &str = "some error occured during analyzing";
pub static IDENTIFIER_NOT_FOUND_ERROR_STR: &str = "identifier not found in current scope";
pub static FUNCTION_NOT_FOUND_ERROR_STR: &str =
    "function with this name not found in current scope";
pub static WRONG_CALL_ARGUMENTS_ERROR_STR: &str = "wrong call arguments";
pub static MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR: &str =
    "identifier defined multiple times in current scope";
pub static MISMATCHED_TYPES_ERROR_STR: &str = "mismatched types";

#[derive(Default)]
pub struct ErrorListener {
    errors: Errors,
}

impl ErrorListener {
    pub fn new() -> Self {
        Self {
            errors: Errors::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn syntax_error(&mut self, message: &str, location: Location) {
        self.errors
            .push(format!("Syntax Error at [{}]: {}", location, message));
    }

    pub fn semantic_error(&mut self, message: &str, scope: &analyzer::Scope) {
        self.errors
            .push(format!("Semantic Error at {:?}: {}", scope, message));
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
