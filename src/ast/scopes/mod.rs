use crate::ast::Ast;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub statements: Vec<Ast>,
    pub is_global: bool,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            is_global: false,
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
