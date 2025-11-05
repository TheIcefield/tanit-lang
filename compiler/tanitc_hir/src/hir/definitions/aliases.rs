use tanitc_attributes::Publicity;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::hir::{definitions::Definition, types::TypeSpec, Hir};

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

impl From<AliasDef> for Hir {
    fn from(value: AliasDef) -> Self {
        Self::Definition(Definition::Alias(value))
    }
}
