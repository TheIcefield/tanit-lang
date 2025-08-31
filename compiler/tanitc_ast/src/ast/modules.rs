use tanitc_attributes::{Publicity, Safety};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::ast::{blocks::Block, Ast};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModuleAttributes {
    pub publicity: Publicity,
    pub safety: Safety,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ModuleDef {
    pub location: Location,
    pub attributes: ModuleAttributes,
    pub identifier: Ident,
    pub is_external: bool,
    pub body: Box<Block>,
}

impl From<ModuleDef> for Ast {
    fn from(value: ModuleDef) -> Self {
        Self::ModuleDef(value)
    }
}
