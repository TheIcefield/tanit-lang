use std::collections::BTreeMap;

use tanitc_ast::ast::aliases::AliasDef;
use tanitc_attributes::Mutability;
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{AliasDefData, Entry, SymbolKind},
    type_info::TypeInfo,
};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        if self.has_symbol(alias_def.identifier) {
            return Err(Message::multiple_ids(
                alias_def.location,
                alias_def.identifier,
            ));
        }

        self.add_symbol(Entry {
            name: alias_def.identifier,
            is_static: true,
            kind: SymbolKind::AliasDef(AliasDefData {
                ty: alias_def.value.get_type(),
            }),
        });

        Ok(())
    }

    pub fn get_alias_def_type(&self, alias_def: &AliasDef) -> TypeInfo {
        TypeInfo {
            ty: alias_def.value.get_type(),
            mutability: Mutability::default(),
            members: BTreeMap::new(),
        }
    }

    pub fn find_alias_value(&self, alias_type: &Type) -> Option<Type> {
        let Type::Custom(id) = alias_type else {
            return None;
        };

        let type_id = Ident::from(id.clone());

        let entry = self.table.lookup(type_id)?;

        let SymbolKind::AliasDef(alias_data) = &entry.kind else {
            return None;
        };

        let Some(alias_to) = self.find_alias_value(&alias_data.ty) else {
            return Some(alias_data.ty.clone());
        };

        Some(alias_to)
    }
}
