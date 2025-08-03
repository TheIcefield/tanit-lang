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
        if self.has_symbol(struct_def.identifier) {
            return Err(Message::multiple_ids(
                struct_def.location,
                struct_def.identifier,
            ));
        }

        for internal in struct_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_ty) in struct_def.fields.iter() {
            fields.insert(
                *field_id,
                StructFieldData {
                    ty: field_ty.ty.get_type(),
                },
            );
        }

        self.add_symbol(Entry {
            name: struct_def.identifier,
            is_static: true,
            kind: SymbolKind::from(StructDefData { fields }),
        });

        Ok(())
    }

    pub fn access_struct_def(
        &mut self,
        struct_name: Ident,
        struct_data: &StructDefData,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value = Box::new(node.clone());

        let struct_comps = &struct_data.fields;

        if let Ast::Value(Value {
            kind:
                ValueKind::Struct {
                    components: value_comps,
                    ..
                },
            ..
        }) = value.as_mut()
        {
            if let Err(mut msg) =
                self.check_struct_components(value_comps, struct_name, struct_comps)
            {
                msg.location = node.location();
                return Err(msg);
            }
        } else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected ValueKind::Struct, actually: {node:?}"),
            ));
        }

        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value,
                ty: Type::Custom(struct_name.to_string()),
            },
        };

        Ok(Some(node))
    }
}
