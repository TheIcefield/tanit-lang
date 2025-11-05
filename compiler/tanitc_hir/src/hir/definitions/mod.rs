pub mod aliases;
pub mod enums;
pub mod externs;
pub mod functions;
pub mod methods;
pub mod modules;
pub mod structs;
pub mod unions;
pub mod variables;
pub mod variants;

use {
    crate::visitor::{Visitor, VisitorMut},
    aliases::AliasDef,
    enums::EnumDef,
    externs::ExternDef,
    functions::FunctionDef,
    methods::ImplDef,
    modules::ModuleDef,
    structs::StructDef,
    tanitc_lexer::location::Location,
    tanitc_messages::Message,
    unions::UnionDef,
    variables::VariableDef,
    variants::VariantDef,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Module(ModuleDef),
    Struct(StructDef),
    Union(UnionDef),
    Variant(VariantDef),
    Impl(ImplDef),
    Enum(EnumDef),
    Func(FunctionDef),
    Variable(VariableDef),
    Alias(AliasDef),
    Extern(ExternDef),
}

impl Definition {
    pub fn accept(&self, visitor: &mut dyn Visitor) -> Result<(), Message> {
        match self {
            Self::Alias(node) => visitor.visit_alias_def(node),
            Self::Enum(node) => visitor.visit_enum_def(node),
            Self::Extern(node) => visitor.visit_extern_def(node),
            Self::Func(node) => visitor.visit_func_def(node),
            Self::Impl(node) => visitor.visit_impl_def(node),
            Self::Module(node) => visitor.visit_module_def(node),
            Self::Struct(node) => visitor.visit_struct_def(node),
            Self::Union(node) => visitor.visit_union_def(node),
            Self::Variable(node) => visitor.visit_variable_def(node),
            Self::Variant(node) => visitor.visit_variant_def(node),
        }
    }

    pub fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) -> Result<(), Message> {
        match self {
            Self::Alias(node) => visitor.visit_alias_def(node),
            Self::Enum(node) => visitor.visit_enum_def(node),
            Self::Extern(node) => visitor.visit_extern_def(node),
            Self::Func(node) => visitor.visit_func_def(node),
            Self::Impl(node) => visitor.visit_impl_def(node),
            Self::Module(node) => visitor.visit_module_def(node),
            Self::Struct(node) => visitor.visit_struct_def(node),
            Self::Union(node) => visitor.visit_union_def(node),
            Self::Variable(node) => visitor.visit_variable_def(node),
            Self::Variant(node) => visitor.visit_variant_def(node),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Self::Alias(node) => node.location,
            Self::Enum(node) => node.location,
            Self::Extern(node) => node.location,
            Self::Func(node) => node.location,
            Self::Impl(node) => node.location,
            Self::Module(node) => node.location,
            Self::Struct(node) => node.location,
            Self::Union(node) => node.location,
            Self::Variable(node) => node.location,
            Self::Variant(node) => node.location,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Alias(_) => "alias definition",
            Self::Enum(_) => "enum definition",
            Self::Extern(_) => "extern definition",
            Self::Func(_) => "function definition",
            Self::Impl(_) => "impl definition",
            Self::Module(_) => "module definition",
            Self::Struct(_) => "struct definition",
            Self::Union(_) => "union definition",
            Self::Variable(_) => "variable definition",
            Self::Variant(_) => "variant definition",
        }
    }
}
