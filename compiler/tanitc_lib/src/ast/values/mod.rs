use crate::ast::{identifiers::Identifier, Ast};

use tanitc_lexer::location::Location;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serializer;

#[derive(Clone, PartialEq)]
pub enum CallParam {
    Notified(Identifier, Box<Ast>),
    Positional(usize, Box<Ast>),
}

#[derive(Clone, PartialEq)]
pub enum ValueType {
    Call {
        identifier: Identifier,
        arguments: Vec<CallParam>,
    },
    Struct {
        identifier: Identifier,
        components: Vec<(Identifier, Ast)>,
    },
    Tuple {
        components: Vec<Ast>,
    },
    Array {
        components: Vec<Ast>,
    },
    Identifier(Identifier),
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
