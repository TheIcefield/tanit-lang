use tanitc_ast::ast::{
    expressions::{Expression, ExpressionKind},
    unions::UnionDef,
    values::ValueKind,
    Ast,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{Entry, StructFieldData, SymbolKind, UnionDefData};
use tanitc_ty::Type;

use std::collections::BTreeMap;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        if self.has_symbol(union_def.identifier) {
            return Err(Message::multiple_ids(
                union_def.location,
                union_def.identifier,
            ));
        }

        for internal in union_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_ty) in union_def.fields.iter() {
            fields.insert(
                *field_id,
                StructFieldData {
                    ty: field_ty.ty.get_type(),
                },
            );
        }

        self.add_symbol(Entry {
            name: union_def.identifier,
            is_static: true,
            kind: SymbolKind::from(UnionDefData { fields }),
        });

        Ok(())
    }

    pub fn access_union_def(
        &mut self,
        union_name: Ident,
        union_data: &UnionDefData,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value_clone = Box::new(node.clone());

        let union_comps = &union_data.fields;

        let Ast::Value(value) = value_clone.as_mut() else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected Ast::Value, actually: {}", node.name()),
            ));
        };

        let ValueKind::Struct {
            identifier: union_id,
            components: value_comps,
        } = &mut value.kind
        else {
            return Err(Message::unreachable(
                value.location,
                format!("expected ValueKind::Struct, actually: {:?}", value.kind),
            ));
        };

        if let Err(mut msg) = self.check_union_components(value_comps, union_name, union_comps) {
            msg.location = node.location();
            return Err(msg);
        }

        value.kind = ValueKind::Struct {
            identifier: *union_id,
            components: std::mem::take(value_comps),
        };

        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value_clone,
                ty: Type::Custom(union_name.to_string()),
            },
        };

        Ok(Some(node))
    }
}
