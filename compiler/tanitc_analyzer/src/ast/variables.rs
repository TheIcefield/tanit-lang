use tanitc_ast::ast::{variables::VariableDef, Ast};
use tanitc_lexer::location::Location;
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

    fn check_ref_coerce_to_ptr(
        &self,
        src_type: &Type,
        dst_type: &Type,
        location: Location,
    ) -> Result<(), Message> {
        let Type::Ref { ref_to, .. } = src_type else {
            return Err(Message::unreachable(
                location,
                format!("dst_type expected to be reference, actually: {src_type}"),
            ));
        };

        let Type::Ptr(ptr_to) = dst_type else {
            return Err(Message::unreachable(
                location,
                format!("src_type expected to be pointer, actually: {dst_type}"),
            ));
        };

        self.compare_types(ref_to, ptr_to.as_ref(), location)
    }

    // Returns true if src_type can be coerced to dst_type, otherwise - false
    fn try_coerce(&self, src_type: &Type, dst_type: &Type, location: Location) -> bool {
        if src_type.is_reference() && dst_type.is_pointer() {
            return self
                .check_ref_coerce_to_ptr(src_type, dst_type, location)
                .is_ok();
        }

        false
    }

    fn compare_types(
        &self,
        lhs_type: &Type,
        rhs_type: &Type,
        location: Location,
    ) -> Result<(), Message> {
        let mut alias_to = self.find_alias_value(lhs_type);

        if lhs_type == rhs_type {
            alias_to = None;
        }

        if alias_to.is_none()
            && lhs_type != rhs_type
            && !self.try_coerce(rhs_type, lhs_type, location)
        {
            return Err(Message {
                    location,
                    text: format!(
                        "Cannot perform operation on objects with different types: {lhs_type} and {rhs_type}",
                    ),
                });
        } else if alias_to.as_ref().is_some_and(|ty| rhs_type != ty)
            && !self.try_coerce(rhs_type, lhs_type, location)
        {
            return Err(Message {
                    location,
                    text: format!(
                        "Cannot perform operation on objects with different types: {lhs_type} (aka: {}) and {rhs_type}",
                        alias_to.unwrap()
                    ),
                });
        }

        Ok(())
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
            lhs.var_type.ty = rhs_type.ty.clone();
        }

        let var_type = lhs.var_type.get_type();
        self.compare_types(&var_type, &rhs_type.ty, lhs.location)?;

        self.add_symbol(Entry {
            name: variable_name,
            is_static: false,
            kind: SymbolKind::from(VarDefData {
                storage: VarStorageType::Auto,
                var_type,
                mutability: lhs.mutability,
                is_initialization: true,
            }),
        });

        Ok(())
    }
}
