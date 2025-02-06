use crate::ast::Ast;

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub enum CallParam {
    Notified(Ident, Box<Ast>),
    Positional(usize, Box<Ast>),
}

#[derive(Clone, PartialEq)]
pub enum ValueType {
    Call {
        identifier: Ident,
        arguments: Vec<CallParam>,
    },
    Struct {
        identifier: Ident,
        components: Vec<(Ident, Ast)>,
    },
    Tuple {
        components: Vec<Ast>,
    },
    Array {
        components: Vec<Ast>,
    },
    Identifier(Ident),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

#[derive(Clone, PartialEq)]
pub struct Value {
    pub location: Location,
    pub value: ValueType,
}

impl From<Value> for Ast {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}
