use tanitc_attributes::{Publicity, Safety};
use tanitc_messages::Message;

use crate::{
    ast::{
        aliases::AliasDef, blocks::Block, branches::Branch, control_flows::ControlFlow,
        enums::EnumDef, expressions::Expression, externs::ExternDef, functions::FunctionDef,
        methods::ImplDef, modules::ModuleDef, structs::StructDef, types::TypeSpec,
        unions::UnionDef, uses::Use, values::Value, variables::VariableDef, variants::VariantDef,
    },
    visitor::VisitorMut,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedAttributes {
    pub safety: Option<Safety>,
    pub publicity: Option<Publicity>,
}

pub struct AttributesApply {
    pub attrs: ParsedAttributes,
}

impl VisitorMut for AttributesApply {
    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        alias_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_use(&mut self, _u: &mut Use) -> Result<(), Message> {
        Ok(())
    }

    fn visit_value(&mut self, _val: &mut Value) -> Result<(), Message> {
        Ok(())
    }

    fn visit_branch(&mut self, _branch: &mut Branch) -> Result<(), Message> {
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message> {
        block.attributes.safety = self.attrs.safety.unwrap_or_default();
        Ok(())
    }

    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        enum_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        func_def.attributes.safety = self.attrs.safety.unwrap_or_default();
        func_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_extern_def(&mut self, _extern_def: &mut ExternDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_type_spec(&mut self, _type_spec: &mut TypeSpec) -> Result<(), Message> {
        Ok(())
    }

    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        union_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_expression(&mut self, _expr: &mut Expression) -> Result<(), Message> {
        Ok(())
    }

    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        module_def.attributes.safety = self.attrs.safety.unwrap_or_default();
        module_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        struct_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        variant_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_impl_def(&mut self, _impl_def: &mut ImplDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_control_flow(&mut self, _cf: &mut ControlFlow) -> Result<(), Message> {
        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        var_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }
}
