use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::Ast;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum UseIdentifier {
    #[default]
    BuiltInSelf,
    BuiltInCrate,
    BuiltInSuper,
    BuiltInAll,
    Identifier(Ident),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Use {
    pub location: Location,
    pub identifier: Vec<UseIdentifier>,
}

impl From<Use> for Ast {
    fn from(value: Use) -> Self {
        Self::Use(value)
    }
}
