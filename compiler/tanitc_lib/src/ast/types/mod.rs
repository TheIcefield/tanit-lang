use crate::ast::Ast;

use tanitc_lexer::location::Location;
use tanitc_ty::Type;

pub mod analyzer;
pub mod codegen;
pub mod parser;
pub mod serialyzer;

#[derive(Default, Clone, Copy, PartialEq)]
pub struct MetaInfo {
    pub is_mut: bool,
}

#[derive(Default, Clone, PartialEq)]
pub struct TypeSpec {
    pub location: Location,
    pub info: MetaInfo,
    pub ty: Type,
}

impl TypeSpec {
    pub fn get_type(&self) -> Type {
        self.ty.clone()
    }

    pub fn get_c_type(&self) -> String {
        self.ty.get_c_type()
    }
}

impl From<TypeSpec> for Ast {
    fn from(value: TypeSpec) -> Self {
        Self::TypeSpec(value)
    }
}
