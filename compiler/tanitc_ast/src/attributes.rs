use std::fmt::Display;

use tanitc_messages::Message;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attributes {
    pub safety: Option<Safety>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    #[default]
    Safe,
    Unsafe,
}

impl Display for Safety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Safe => write!(f, "safe")?,
            Self::Unsafe => write!(f, "unsafe")?,
        }

        Ok(())
    }
}

pub struct AttributesApply {
    pub attrs: Attributes,
}

impl super::VisitorMut for AttributesApply {
    fn visit_alias_def(&mut self, _alias_def: &mut crate::AliasDef) -> Result<(), Message> {
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
        block.attrs = self.attrs;
        Ok(())
    }

    fn visit_enum_def(&mut self, _enum_def: &mut crate::EnumDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &mut crate::FunctionDef) -> Result<(), Message> {
        func_def.attrs = self.attrs;
        Ok(())
    }

    fn visit_type_spec(&mut self, _type_spec: &mut crate::TypeSpec) -> Result<(), Message> {
        Ok(())
    }

    fn visit_union_def(&mut self, _union_def: &mut crate::UnionDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_expression(&mut self, _expr: &mut crate::Expression) -> Result<(), Message> {
        Ok(())
    }

    fn visit_module_def(&mut self, module_def: &mut crate::ModuleDef) -> Result<(), Message> {
        module_def.attrs = self.attrs;
        Ok(())
    }

    fn visit_struct_def(&mut self, _struct_def: &mut crate::StructDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_variant_def(&mut self, _variant_def: &mut crate::VariantDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_control_flow(&mut self, _cf: &mut crate::ControlFlow) -> Result<(), Message> {
        Ok(())
    }

    fn visit_variable_def(&mut self, _var_def: &mut crate::VariableDef) -> Result<(), Message> {
        Ok(())
    }
}
