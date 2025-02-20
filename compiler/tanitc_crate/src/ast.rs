use tanitc_ast::{
    AliasDef, AstVisitor, Block, Branch, ControlFlow, EnumDef, Expression, FunctionDef, ModuleDef,
    StructDef, TypeSpec, Value, VariableDef, VariantDef,
};
use tanitc_messages::Message;

use crate::Unit;

pub struct ModuleSearcher {
    pub current_path: String,
    pub subunits: Vec<Unit>,
}

impl ModuleSearcher {
    fn add_subunit(&mut self, unit: Unit) {
        self.subunits.push(unit);
    }
}

impl AstVisitor for ModuleSearcher {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if module_def.is_external {
            let name: String = module_def.identifier.into();

            let mut path = self
                .current_path
                .chars()
                .rev()
                .collect::<String>()
                .splitn(2, '/')
                .collect::<Vec<&str>>()[1]
                .chars()
                .rev()
                .collect::<String>();

            path.push('/');
            path.push_str(&name);

            let mut unit_exists: bool;

            {
                let mut path = path.clone();
                path.push_str(".tt");

                unit_exists = std::path::Path::new(&path).exists();
                if unit_exists {
                    self.add_subunit(
                        Unit::builder()
                            .set_name(name.clone())
                            .set_path(path)
                            .build(),
                    );
                }
            }

            if !unit_exists {
                let mut path = path.clone();
                path.push_str("/mod.tt");

                unit_exists = std::path::Path::new(&path).exists();
                if unit_exists {
                    self.add_subunit(
                        Unit::builder()
                            .set_name(name.clone())
                            .set_path(path)
                            .build(),
                    );
                }
            }

            if !unit_exists {
                return Err(Message::new(
                    module_def.location,
                    &format!("Unit {name} not found"),
                ));
            }

            return Ok(());
        } else if let Some(body) = &mut module_def.body {
            self.visit_block(body)?;
        }

        Ok(())
    }

    fn visit_struct_def(&mut self, _struct_def: &mut StructDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_variant_def(&mut self, _variant_def: &mut VariantDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_enum_def(&mut self, _enum_def: &mut EnumDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_func_def(&mut self, _func_def: &mut FunctionDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_variable_def(&mut self, _var_def: &mut VariableDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_alias_def(&mut self, _alias_def: &mut AliasDef) -> Result<(), Message> {
        Ok(())
    }

    fn visit_expression(&mut self, _expr: &mut Expression) -> Result<(), Message> {
        Ok(())
    }

    fn visit_branch(&mut self, _branch: &mut Branch) -> Result<(), Message> {
        Ok(())
    }

    fn visit_control_flow(&mut self, _cf: &mut ControlFlow) -> Result<(), Message> {
        Ok(())
    }

    fn visit_type_spec(&mut self, _type_spec: &mut TypeSpec) -> Result<(), Message> {
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message> {
        for stmt in block.statements.iter_mut() {
            self.visit(stmt)?;
        }

        Ok(())
    }

    fn visit_value(&mut self, _val: &mut Value) -> Result<(), Message> {
        Ok(())
    }
}
