use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;

use crate::ast::Ast;

#[derive(Debug, Clone, PartialEq)]
pub enum CallArgKind {
    Notified(Ident, Box<Ast>),
    Positional(usize, Box<Ast>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub location: Location,
    pub identifier: Option<Ident>,
    pub kind: CallArgKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Call {
        identifier: Ident,
        arguments: Vec<CallArg>,
    },
    Struct {
        name: Name,
        components: Vec<(Name, Ast)>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub location: Location,
    pub kind: ValueKind,
}

impl From<Value> for Ast {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}
