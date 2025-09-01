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
        if self.has_symbol(union_def.name.id) {
            return Err(Message::multiple_ids(union_def.location, union_def.name.id));
        }

        union_def.name.prefix = self.table.get_id();

        for internal in union_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let mut fields = BTreeMap::<Ident, StructFieldData>::new();
        for (field_id, field_info) in union_def.fields.iter() {
            let Some(ty) = self.table.lookup_type(&field_info.ty.ty) else {
                self.error(Message::undefined_type(
                    field_info.ty.location,
                    &field_info.ty.ty.as_str(),
                ));
                continue;
            };

            fields.insert(
                *field_id,
                StructFieldData {
                    struct_name: union_def.name,
                    ty: ty.ty,
                },
            );
        }

        self.add_symbol(Entry {
            name: union_def.name.id,
            is_static: true,
            kind: SymbolKind::from(UnionDefData {
                name: union_def.name,
                fields,
            }),
        });

        Ok(())
    }

    pub fn access_union_def(
        &mut self,
        union_data: &UnionDefData,
        node: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut value_clone = Box::new(node.clone());

        let Ast::Value(value) = value_clone.as_mut() else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected Ast::Value, actually: {}", node.name()),
            ));
        };

        let ValueKind::Struct {
            name: union_name,
            components: value_comps,
        } = &mut value.kind
        else {
            return Err(Message::unreachable(
                value.location,
                format!("expected ValueKind::Struct, actually: {:?}", value.kind),
            ));
        };

        if let Err(mut msg) = self.check_union_components(value_comps, union_data) {
            msg.location = node.location();
            return Err(msg);
        }

        *union_name = union_data.name;
        let node = Expression {
            location: node.location(),
            kind: ExpressionKind::Term {
                node: value_clone,
                ty: Type::Custom(union_data.name),
            },
        };

        Ok(Some(node))
    }
}

#[cfg(test)]
mod tests {

    use tanitc_ast::ast::{
        blocks::Block,
        expressions::{Expression, ExpressionKind},
        functions::FunctionDef,
        types::TypeSpec,
        unions::{UnionDef, UnionFieldInfo, UnionFields},
        values::{Value, ValueKind},
        variables::VariableDef,
        Ast,
    };
    use tanitc_attributes::Safety;
    use tanitc_ident::{Ident, Name};
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    use crate::Analyzer;

    const FIELD_NAME: &str = "field";

    fn get_union(name: &str) -> UnionDef {
        let mut fields = UnionFields::new();
        fields.insert(
            Ident::from(FIELD_NAME.to_string()),
            UnionFieldInfo {
                ty: TypeSpec {
                    ty: Type::I32,
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        UnionDef {
            name: Name::from(name.to_string()),
            fields,
            ..Default::default()
        }
    }

    fn get_func(name: &str, statements: Vec<Ast>) -> FunctionDef {
        FunctionDef {
            name: Name::from(name.to_string()),
            body: Some(Box::new(Block {
                statements,
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    fn get_var(name: &str, ty: Type) -> VariableDef {
        VariableDef {
            identifier: Ident::from(name.to_string()),
            var_type: TypeSpec {
                ty,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn get_access(var_name: &str) -> Expression {
        Expression {
            location: Location::new(),
            kind: ExpressionKind::Get {
                lhs: Box::new(Ast::from(Value {
                    kind: ValueKind::Identifier(Ident::from(var_name.to_string())),
                    location: Location::new(),
                })),
                rhs: Box::new(Ast::from(Value {
                    kind: ValueKind::Identifier(Ident::from(FIELD_NAME.to_string())),
                    location: Location::new(),
                })),
            },
        }
    }

    #[test]
    fn union_unsafe_access_good_test() {
        const UNION_NAME: &str = "UnionName";
        const FUNC_NAME: &str = "safe_func";
        const VAR_NAME: &str = "my_union";

        let mut func_def = get_func(
            FUNC_NAME,
            vec![
                get_var(VAR_NAME, Type::Custom(Name::from(UNION_NAME.to_string()))).into(),
                get_access(VAR_NAME).into(),
            ],
        );
        func_def.attributes.safety = Safety::Unsafe;

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_union(UNION_NAME).into(), func_def.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:?}", analyzer.get_errors());
        }
    }

    #[test]
    fn union_safe_access_bad_test() {
        const UNION_NAME: &str = "UnionName";
        const FUNC_NAME: &str = "safe_func";
        const VAR_NAME: &str = "my_union";

        const EXPECTED_ERR: &str = "Semantic error: Access to union field is unsafe and requires an unsafe function or block";

        let func_def = get_func(
            FUNC_NAME,
            vec![
                get_var(VAR_NAME, Type::Custom(Name::from(UNION_NAME.to_string()))).into(),
                get_access(VAR_NAME).into(),
            ],
        );

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_union(UNION_NAME).into(), func_def.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
