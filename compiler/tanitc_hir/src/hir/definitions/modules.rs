use tanitc_attributes::{Publicity, Safety};
use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::{blocks::Block, definitions::Definition, Hir};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModuleAttributes {
    pub publicity: Publicity,
    pub safety: Safety,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDefBody {
    Internal(Box<Block>),
    External(Box<Hir>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDef {
    pub location: Location,
    pub attributes: ModuleAttributes,
    pub name: NameSpec,
    pub body: ModuleDefBody,
}

impl From<ModuleDef> for Hir {
    fn from(value: ModuleDef) -> Self {
        Self::Definition(Definition::Module(value))
    }
}
