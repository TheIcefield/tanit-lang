use tanitc_lexer::location::Location;

use crate::hir::{expressions::Expression, types::TypeSpec};

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionExpr {
    pub location: Location,
    pub expr: Box<Expression>,
    pub ty: TypeSpec,
}
