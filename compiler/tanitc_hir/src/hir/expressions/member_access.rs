use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::hir::expressions::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccessExpr {
    pub location: Location,
    pub lhs: Box<Expression>,
    pub id: Ident,
}
