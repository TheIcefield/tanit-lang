use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{types::TypeSpec, Ast};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AliasAttributes {
    pub publicity: Publicity,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AliasDef {
    pub location: Location,
    pub attributes: AliasAttributes,
    pub identifier: Ident,
    pub value: TypeSpec,
}

impl From<AliasDef> for Ast {
    fn from(value: AliasDef) -> Self {
        Self::AliasDef(value)
    }
}
