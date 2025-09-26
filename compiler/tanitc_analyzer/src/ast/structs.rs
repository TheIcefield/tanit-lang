use std::collections::BTreeMap;

use tanitc_ast::ast::{
    expressions::{Expression, ExpressionKind},
    structs::StructDef,
    values::{Value, ValueKind},
    Ast,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{Entry, StructDefData, StructFieldData, SymbolKind};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        if self.has_symbol(struct_def.name.id) {
            return Err(Message::multiple_ids(
                &struct_def.location,
                struct_def.name.id,
            ));
        }

        struct_def.name.prefix = self.table.get_id();

        for internal in struct_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_info) in struct_def.fields.iter() {
            let Some(ty) = self.table.lookup_type(&field_info.ty.ty) else {
                self.error(Message::undefined_type(
                    &field_info.ty.location,
                    &field_info.ty.ty.as_str(),
                ));
                continue;
            };

            fields.insert(
                *field_id,
                StructFieldData {
                    struct_name: struct_def.name,
                    ty: ty.ty,
                },
            );
        }

        self.add_symbol(Entry {
            name: struct_def.name.id,
            is_static: true,
            kind: SymbolKind::from(StructDefData {
                name: struct_def.name,
                fields,
            }),
        });

        Ok(())
    }

    pub fn access_struct_def(
        &mut self,
        struct_data: &StructDefData,
        node: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value = Box::new(node.clone());
        let location = node.location();

        let Ast::Value(Value {
            kind:
                ValueKind::Struct {
                    name: struct_name,
                    components: value_comps,
                },
            ..
        }) = value.as_mut()
        else {
            return Err(Message::unreachable(
                &node.location(),
                format!("expected ValueKind::Struct, actually: {node:?}"),
            ));
        };

        self.check_struct_value_components(value_comps, struct_data, &location)?;

        *struct_name = struct_data.name;
        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value,
                ty: Type::Custom(struct_data.name),
            },
        };

        Ok(Some(node))
    }
}
