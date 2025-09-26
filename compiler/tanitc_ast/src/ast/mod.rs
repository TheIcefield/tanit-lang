use std::fmt::Debug;

use tanitc_lexer::location::Location;
use tanitc_messages::Message;

pub mod aliases;
pub mod blocks;
pub mod branches;
pub mod control_flows;
pub mod enums;
pub mod expressions;
pub mod externs;
pub mod functions;
pub mod methods;
pub mod modules;
pub mod structs;
pub mod types;
pub mod unions;
pub mod uses;
pub mod values;
pub mod variables;
pub mod variants;

use crate::{
    ast::{
        aliases::AliasDef, blocks::Block, branches::Branch, control_flows::ControlFlow,
        enums::EnumDef, expressions::Expression, externs::ExternDef, functions::FunctionDef,
        methods::ImplDef, modules::ModuleDef, structs::StructDef, types::TypeSpec,
        unions::UnionDef, uses::Use, values::Value, variables::VariableDef, variants::VariantDef,
    },
    attributes,
    visitor::{Visitor, VisitorMut},
};

#[derive(Clone, PartialEq)]
pub enum Ast {
    ModuleDef(ModuleDef),
    StructDef(StructDef),
    UnionDef(UnionDef),
    VariantDef(VariantDef),
    ImplDef(ImplDef),
    EnumDef(EnumDef),
    FuncDef(FunctionDef),
    VariableDef(VariableDef),
    AliasDef(AliasDef),
    ExternDef(ExternDef),
    Expression(Expression),
    BranchStmt(Branch),
    ControlFlow(ControlFlow),
    TypeSpec(TypeSpec),
    Use(Use),
    Block(Block),
    Value(Value),
}

impl Ast {
    pub fn accept(&self, visitor: &mut dyn Visitor) -> Result<(), Message> {
        match self {
            Self::ModuleDef(node) => visitor.visit_module_def(node),
            Self::StructDef(node) => visitor.visit_struct_def(node),
            Self::UnionDef(node) => visitor.visit_union_def(node),
            Self::VariantDef(node) => visitor.visit_variant_def(node),
            Self::ImplDef(node) => visitor.visit_impl_def(node),
            Self::EnumDef(node) => visitor.visit_enum_def(node),
            Self::FuncDef(node) => visitor.visit_func_def(node),
            Self::VariableDef(node) => visitor.visit_variable_def(node),
            Self::AliasDef(node) => visitor.visit_alias_def(node),
            Self::ExternDef(node) => visitor.visit_extern_def(node),
            Self::Expression(node) => visitor.visit_expression(node),
            Self::BranchStmt(node) => visitor.visit_branch(node),
            Self::ControlFlow(node) => visitor.visit_control_flow(node),
            Self::TypeSpec(node) => visitor.visit_type_spec(node),
            Self::Use(node) => visitor.visit_use(node),
            Self::Block(node) => visitor.visit_block(node),
            Self::Value(node) => visitor.visit_value(node),
        }
    }

    pub fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) -> Result<(), Message> {
        match self {
            Self::ModuleDef(node) => visitor.visit_module_def(node),
            Self::StructDef(node) => visitor.visit_struct_def(node),
            Self::UnionDef(node) => visitor.visit_union_def(node),
            Self::VariantDef(node) => visitor.visit_variant_def(node),
            Self::ImplDef(node) => visitor.visit_impl_def(node),
            Self::EnumDef(node) => visitor.visit_enum_def(node),
            Self::FuncDef(node) => visitor.visit_func_def(node),
            Self::VariableDef(node) => visitor.visit_variable_def(node),
            Self::AliasDef(node) => visitor.visit_alias_def(node),
            Self::ExternDef(node) => visitor.visit_extern_def(node),
            Self::Expression(node) => visitor.visit_expression(node),
            Self::BranchStmt(node) => visitor.visit_branch(node),
            Self::ControlFlow(node) => visitor.visit_control_flow(node),
            Self::TypeSpec(node) => visitor.visit_type_spec(node),
            Self::Use(node) => visitor.visit_use(node),
            Self::Block(node) => visitor.visit_block(node),
            Self::Value(node) => visitor.visit_value(node),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Self::ModuleDef(node) => node.location.clone(),
            Self::StructDef(node) => node.location.clone(),
            Self::UnionDef(node) => node.location.clone(),
            Self::VariantDef(node) => node.location.clone(),
            Self::ImplDef(node) => node.location.clone(),
            Self::EnumDef(node) => node.location.clone(),
            Self::FuncDef(node) => node.location.clone(),
            Self::VariableDef(node) => node.location.clone(),
            Self::AliasDef(node) => node.location.clone(),
            Self::ExternDef(node) => node.location.clone(),
            Self::Expression(node) => node.location.clone(),
            Self::BranchStmt(node) => node.location.clone(),
            Self::ControlFlow(node) => node.location.clone(),
            Self::TypeSpec(node) => node.location.clone(),
            Self::Use(node) => node.location.clone(),
            Self::Block(node) => node.location.clone(),
            Self::Value(node) => node.location.clone(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ModuleDef(_) => "module definition",
            Self::StructDef(_) => "struct definition",
            Self::UnionDef(_) => "union definition",
            Self::VariantDef(_) => "variant definition",
            Self::ImplDef(_) => "impl definition",
            Self::EnumDef(_) => "enum definition",
            Self::FuncDef(_) => "function definition",
            Self::VariableDef(_) => "variable definition",
            Self::AliasDef(_) => "alias definition",
            Self::ExternDef(_) => "extern definition",
            Self::Expression(_) => "expression",
            Self::BranchStmt(_) => "branching",
            Self::ControlFlow(cf) => cf.kind.to_str(),
            Self::TypeSpec(_) => "type specification",
            Self::Use(_) => "using",
            Self::Block(_) => "block",
            Self::Value(_) => "value",
        }
    }

    pub fn apply_attributes(&mut self, attrs: attributes::ParsedAttributes) -> Result<(), Message> {
        let mut visitor = attributes::AttributesApply { attrs };
        self.accept_mut(&mut visitor)
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::Block(Block::default())
    }
}

impl Debug for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleDef(node) => node.fmt(f),
            Self::StructDef(node) => node.fmt(f),
            Self::UnionDef(node) => node.fmt(f),
            Self::VariantDef(node) => node.fmt(f),
            Self::ImplDef(node) => node.fmt(f),
            Self::EnumDef(node) => node.fmt(f),
            Self::FuncDef(node) => node.fmt(f),
            Self::VariableDef(node) => node.fmt(f),
            Self::AliasDef(node) => node.fmt(f),
            Self::ExternDef(node) => node.fmt(f),
            Self::Expression(node) => node.fmt(f),
            Self::BranchStmt(node) => node.fmt(f),
            Self::ControlFlow(node) => node.fmt(f),
            Self::TypeSpec(node) => node.fmt(f),
            Self::Use(node) => node.fmt(f),
            Self::Block(node) => node.fmt(f),
            Self::Value(node) => node.fmt(f),
        }
    }
}
