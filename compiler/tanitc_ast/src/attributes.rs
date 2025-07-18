use tanitc_attributes::{Publicity, Safety};
use tanitc_messages::Message;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedAttributes {
    pub safety: Option<Safety>,
    pub publicity: Option<Publicity>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StructAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImplAttributes {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VariantAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnionAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnumAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VariableAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AliasAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FieldAttributes {
    pub publicity: Publicity,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModuleAttributes {
    pub publicity: Publicity,
    pub safety: Safety,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FunctionAttributes {
    pub publicity: Publicity,
    pub safety: Safety,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BlockAttributes {
    pub safety: Safety,
}

pub struct AttributesApply {
    pub attrs: ParsedAttributes,
}

impl super::VisitorMut for AttributesApply {
    fn visit_alias_def(&mut self, alias_def: &mut crate::AliasDef) -> Result<(), Message> {
        alias_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_use(&mut self, _u: &mut crate::Use) -> Result<(), Message> {
        Ok(())
    }

    fn visit_value(&mut self, _val: &mut crate::Value) -> Result<(), Message> {
        Ok(())
    }

    fn visit_branch(&mut self, _branch: &mut crate::Branch) -> Result<(), Message> {
        Ok(())
    }

    fn visit_block(&mut self, block: &mut crate::Block) -> Result<(), Message> {
        block.attributes.safety = self.attrs.safety.unwrap_or_default();
        Ok(())
    }

    fn visit_enum_def(&mut self, enum_def: &mut crate::EnumDef) -> Result<(), Message> {
        enum_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &mut crate::FunctionDef) -> Result<(), Message> {
        func_def.attributes.safety = self.attrs.safety.unwrap_or_default();
        func_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_extern_def(&mut self, _extern_def: &mut crate::ExternDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_type_spec(&mut self, _type_spec: &mut crate::TypeSpec) -> Result<(), Message> {
        Ok(())
    }

    fn visit_union_def(&mut self, union_def: &mut crate::UnionDef) -> Result<(), Message> {
        union_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_expression(&mut self, _expr: &mut crate::Expression) -> Result<(), Message> {
        Ok(())
    }

    fn visit_module_def(&mut self, module_def: &mut crate::ModuleDef) -> Result<(), Message> {
        module_def.attributes.safety = self.attrs.safety.unwrap_or_default();
        module_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_struct_def(&mut self, struct_def: &mut crate::StructDef) -> Result<(), Message> {
        struct_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_variant_def(&mut self, variant_def: &mut crate::VariantDef) -> Result<(), Message> {
        variant_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }

    fn visit_impl_def(&mut self, _impl_def: &mut crate::ImplDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_control_flow(&mut self, _cf: &mut crate::ControlFlow) -> Result<(), Message> {
        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &mut crate::VariableDef) -> Result<(), Message> {
        var_def.attributes.publicity = self.attrs.publicity.unwrap_or_default();
        Ok(())
    }
}
