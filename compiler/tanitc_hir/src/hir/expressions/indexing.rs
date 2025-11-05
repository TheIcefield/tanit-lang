use tanitc_lexer::location::Location;

use crate::hir::expressions::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct IndexingExpr {
    pub location: Location,
    pub lhs: Box<Expression>,
    pub index: Box<Expression>,
}
