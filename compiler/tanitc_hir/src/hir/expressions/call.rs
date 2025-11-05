use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::hir::expressions::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct NamedCallArg {
    pub location: Location,
    pub id: Ident,
    pub expr: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PositionalCallArg {
    pub location: Location,
    pub id: usize,
    pub expr: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallArg {
    Notified(NamedCallArg),
    Positional(PositionalCallArg),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub location: Location,
    pub expr: Box<Expression>,
    pub arguments: Vec<CallArg>,
}

impl CallArg {
    pub fn location(&self) -> Location {
        match self {
            Self::Notified(arg) => arg.location,
            Self::Positional(arg) => arg.location,
        }
    }
}
