use tanitc_ast::ast::{types::TypeSpec, variables::VariableDef, Ast};
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{Entry, SymbolKind, VarDefData, VarStorageType},
    type_info::TypeInfo,
};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        if self.has_symbol(var_def.identifier) {
            return Err(Message::multiple_ids(var_def.location, var_def.identifier));
        }

        self.add_symbol(Entry {
            name: var_def.identifier,
            is_static: false,
            kind: SymbolKind::from(VarDefData {
                var_type: var_def.var_type.get_type(),
                mutability: var_def.mutability,
                is_initialization: false,
                storage: VarStorageType::Auto,
            }),
        });

        Ok(())
    }

    pub fn get_var_def_type(&self, var_def: &VariableDef) -> TypeInfo {
        let Some(mut type_info) = self.table.lookup_type(&var_def.var_type.ty) else {
            return TypeInfo {
                ty: var_def.var_type.ty.clone(),
                mutability: var_def.mutability,
                ..Default::default()
            };
        };

        type_info.mutability = var_def.mutability;
        type_info
    }

    pub fn check_variable_def_types(
        &mut self,
        lhs: &mut VariableDef,
        rhs: Option<&mut Ast>,
    ) -> Result<(), Message> {
        let variable_name = lhs.identifier;

        let Some(rhs) = rhs else {
            return Err(Message::from_string(
                lhs.location,
                format!("Variable {variable_name} is defined, but not initialized"),
            ));
        };

        let rhs_type = self.get_type(rhs);

        if self.has_symbol(variable_name) {
            return Err(Message::multiple_ids(rhs.location(), variable_name));
        }

        if Type::Auto == lhs.var_type.get_type() {
            // Use rhs type
            lhs.var_type.ty = rhs_type.ty.clone();
        } else {
            // Analyze specified type
            self.analyze_variable_type(&mut lhs.var_type)?;
            self.compare_types(&lhs.var_type.ty, &rhs_type.ty, lhs.var_type.location)?;
        }

        self.add_symbol(Entry {
            name: variable_name,
            is_static: false,
            kind: SymbolKind::from(VarDefData {
                storage: VarStorageType::Auto,
                var_type: lhs.var_type.get_type(),
                mutability: lhs.mutability,
                is_initialization: true,
            }),
        });

        Ok(())
    }
}

impl Analyzer {
    fn analyze_variable_type(&self, var_type: &mut TypeSpec) -> Result<(), Message> {
        let Some(type_info) = self.table.lookup_type(&var_type.ty) else {
            return Err(Message::undefined_type(
                var_type.location,
                &var_type.ty.as_str(),
            ));
        };

        var_type.ty = type_info.ty;

        Ok(())
    }
}
