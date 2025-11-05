use tanitc_hir::{
    hir::{
        blocks::Block,
        branches::Branch,
        control_flows::ControlFlow,
        definitions::{
            aliases::AliasDef, enums::EnumDef, externs::ExternDef, functions::FunctionDef,
            methods::ImplDef, modules::ModuleDef, structs::StructDef, unions::UnionDef,
            variables::VariableDef, variants::VariantDef,
        },
        expressions::Expression,
        types::TypeSpec,
        uses::Use,
        Hir,
    },
    visitor::Visitor,
};
use tanitc_messages::Message;

use super::CodeGenStream;

pub(crate) mod blocks;
pub(crate) mod branches;
pub(crate) mod control_flows;
pub(crate) mod definitions;
pub(crate) mod expressions;
pub(crate) mod types;
pub(crate) mod uses;

impl Visitor for CodeGenStream<'_> {
    fn visit_module_def(&mut self, module_def: &ModuleDef) -> Result<(), Message> {
        self.generate_module_def(module_def)
            .map_err(|err| Message::codegen_err(module_def.location, err))
    }

    fn visit_struct_def(&mut self, struct_def: &StructDef) -> Result<(), Message> {
        self.generate_struct_def(struct_def)
            .map_err(|err| Message::codegen_err(struct_def.location, err))
    }

    fn visit_union_def(&mut self, union_def: &UnionDef) -> Result<(), Message> {
        self.generate_union_def(union_def)
            .map_err(|err| Message::codegen_err(union_def.location, err))
    }

    fn visit_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), Message> {
        self.generate_variant_def(variant_def)
            .map_err(|err| Message::codegen_err(variant_def.location, err))
    }

    fn visit_impl_def(&mut self, impl_def: &ImplDef) -> Result<(), Message> {
        self.generate_impl_def(impl_def)
            .map_err(|err| Message::codegen_err(impl_def.location, err))
    }

    fn visit_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), Message> {
        self.generate_enum_def(enum_def)
            .map_err(|err| Message::codegen_err(enum_def.location, err))
    }

    fn visit_func_def(&mut self, func_def: &FunctionDef) -> Result<(), Message> {
        self.generate_func_def(func_def, None)
            .map_err(|err| Message::codegen_err(func_def.location, err))
    }

    fn visit_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), Message> {
        self.generate_extern_def(extern_def)
            .map_err(|err| Message::codegen_err(extern_def.location, err))
    }

    fn visit_variable_def(&mut self, var_def: &VariableDef) -> Result<(), Message> {
        self.generate_variable_def(var_def)
            .map_err(|err| Message::codegen_err(var_def.location, err))
    }

    fn visit_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), Message> {
        self.generate_alias_def(alias_def)
            .map_err(|err| Message::codegen_err(alias_def.location, err))
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), Message> {
        self.generate_expression(expr)
            .map_err(|err| Message::codegen_err(expr.location(), err))
    }

    fn visit_branch(&mut self, branch: &Branch) -> Result<(), Message> {
        self.generate_branch(branch)
            .map_err(|err| Message::codegen_err(branch.location(), err))
    }

    fn visit_control_flow(&mut self, cf: &ControlFlow) -> Result<(), Message> {
        self.generate_control_flow(cf)
            .map_err(|err| Message::codegen_err(cf.location, err))
    }

    fn visit_type_spec(&mut self, type_spec: &TypeSpec) -> Result<(), Message> {
        self.generate_type_spec(type_spec)
            .map_err(|err| Message::codegen_err(type_spec.location, err))
    }

    fn visit_use(&mut self, u: &Use) -> Result<(), Message> {
        self.generate_use(u)
            .map_err(|err| Message::codegen_err(u.location, err))
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), Message> {
        self.generate_block(block)
            .map_err(|err| Message::codegen_err(block.location, err))
    }
}

impl CodeGenStream<'_> {
    fn generate(&mut self, node: &Hir) -> std::io::Result<()> {
        match node {
            Hir::Definition(node) => self.generate_definition(node),
            Hir::Expression(node) => self.generate_expression(node),
            Hir::BranchStmt(node) => self.generate_branch(node),
            Hir::ControlFlow(node) => self.generate_control_flow(node),
            Hir::TypeSpec(node) => self.generate_type_spec(node),
            Hir::Use(node) => self.generate_use(node),
            Hir::Block(node) => self.generate_block(node),
        }
    }
}
