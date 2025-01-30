use crate::ast::Ast;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Default, Clone, PartialEq)]
pub struct Scope {
    pub statements: Vec<Ast>,
    pub is_global: bool,
}

impl From<Scope> for Ast {
    fn from(value: Scope) -> Self {
        Self::Scope(value)
    }
}
