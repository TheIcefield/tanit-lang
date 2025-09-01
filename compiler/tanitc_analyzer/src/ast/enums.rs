use std::collections::BTreeMap;

use tanitc_ast::ast::{
    enums::EnumDef,
    expressions::{Expression, ExpressionKind},
    values::{Value, ValueKind},
    Ast,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{Entry, EnumData, EnumDefData, SymbolKind};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        if self.has_symbol(enum_def.name.id) {
            return Err(Message::multiple_ids(enum_def.location, enum_def.name.id));
        }

        enum_def.name.prefix = self.table.get_id();

        let mut counter = 0usize;
        let mut enums = BTreeMap::<Ident, Entry>::new();
        for field in enum_def.fields.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            enums.insert(
                *field.0,
                Entry {
                    name: *field.0,
                    is_static: true,
                    kind: SymbolKind::Enum(EnumData {
                        enum_name: enum_def.name,
                        value: counter,
                    }),
                },
            );

            counter += 1;
        }

        self.add_symbol(Entry {
            name: enum_def.name.id,
            is_static: true,
            kind: SymbolKind::from(EnumDefData {
                name: enum_def.name,
                enums,
            }),
        });

        Ok(())
    }

    pub fn access_enum(
        &mut self,
        enum_data: &EnumData,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let location = rhs.location();

        Ok(Some(Expression {
            location,
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location,
                    kind: ValueKind::Integer(enum_data.value),
                })),
                ty: Type::Custom(enum_data.enum_name),
            },
        }))
    }
}
