use tanitc_lexer::location::Location;
use tanitc_messages::Message;

pub mod definitions;

pub mod blocks;
pub mod branches;
pub mod control_flows;
pub mod expressions;
pub mod type_spec;
pub mod uses;

use crate::{
    attributes,
    hir::{
        blocks::Block, branches::Branch, control_flows::ControlFlow, definitions::Definition,
        expressions::Expression, type_spec::TypeSpec, uses::Use,
    },
    visitor::{Visitor, VisitorMut},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Hir {
    Definition(Definition),
    Expression(Expression),
    BranchStmt(Branch),
    ControlFlow(ControlFlow),
    TypeSpec(TypeSpec),
    Use(Use),
    Block(Block),
}

impl Hir {
    pub fn accept(&self, visitor: &mut dyn Visitor) -> Result<(), Message> {
        match self {
            Self::Definition(definition) => match definition {
                Definition::Alias(node) => visitor.visit_alias_def(node),
                Definition::Enum(node) => visitor.visit_enum_def(node),
                Definition::Extern(node) => visitor.visit_extern_def(node),
                Definition::Func(node) => visitor.visit_func_def(node),
                Definition::Impl(node) => visitor.visit_impl_def(node),
                Definition::Module(node) => visitor.visit_module_def(node),
                Definition::Struct(node) => visitor.visit_struct_def(node),
                Definition::Union(node) => visitor.visit_union_def(node),
                Definition::Variable(node) => visitor.visit_variable_def(node),
                Definition::Variant(node) => visitor.visit_variant_def(node),
            },
            Self::Expression(node) => visitor.visit_expression(node),
            Self::BranchStmt(node) => visitor.visit_branch(node),
            Self::ControlFlow(node) => visitor.visit_control_flow(node),
            Self::TypeSpec(node) => visitor.visit_type_spec(node),
            Self::Use(node) => visitor.visit_use(node),
            Self::Block(node) => visitor.visit_block(node),
        }
    }

    pub fn accept_mut(&mut self, visitor: &mut dyn VisitorMut) -> Result<(), Message> {
        match self {
            Self::Definition(definition) => definition.accept_mut(visitor),
            Self::Expression(node) => visitor.visit_expression(node),
            Self::BranchStmt(node) => visitor.visit_branch(node),
            Self::ControlFlow(node) => visitor.visit_control_flow(node),
            Self::TypeSpec(node) => visitor.visit_type_spec(node),
            Self::Use(node) => visitor.visit_use(node),
            Self::Block(node) => visitor.visit_block(node),
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Self::Definition(node) => node.location(),
            Self::Expression(node) => node.location(),
            Self::BranchStmt(node) => node.location(),
            Self::ControlFlow(node) => node.location,
            Self::TypeSpec(node) => node.location,
            Self::Use(node) => node.location,
            Self::Block(node) => node.location,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Definition(node) => node.kind_str(),
            Self::Expression(_) => "expression",
            Self::BranchStmt(_) => "branching",
            Self::ControlFlow(cf) => cf.kind.to_str(),
            Self::TypeSpec(_) => "type specification",
            Self::Use(_) => "using",
            Self::Block(_) => "block",
        }
    }

    pub fn apply_attributes(&mut self, attrs: attributes::ParsedAttributes) -> Result<(), Message> {
        let mut visitor = attributes::AttributesApply { attrs };
        self.accept_mut(&mut visitor)
    }
}

impl Default for Hir {
    fn default() -> Self {
        Self::Block(Block::default())
    }
}
