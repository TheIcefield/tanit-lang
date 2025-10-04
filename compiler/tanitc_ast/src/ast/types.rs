use tanitc_attributes::Mutability;
use tanitc_lexer::location::Location;
use tanitc_ty::Type;

use crate::ast::Ast;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ParsedTypeInfo {
    pub mutability: Mutability,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TypeSpec {
    pub location: Location,
    pub info: ParsedTypeInfo,
    pub ty: Type,
}

impl TypeSpec {
    pub fn auto() -> Self {
        Self {
            ty: Type::Auto,
            ..Default::default()
        }
    }

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
