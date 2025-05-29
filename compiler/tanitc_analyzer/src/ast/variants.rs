use std::collections::BTreeMap;

use tanitc_ast::{
    variant_utils, Ast, Expression, ExpressionKind, Value, ValueKind, VariantDef, VariantField,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{
    Entry, StructFieldData, SymbolKind, VariantData, VariantDefData, VariantKind,
    VariantStructKind, VariantTupleKind,
};
use tanitc_ty::Type;

use crate::Analyzer;

// Variant
impl Analyzer {
    pub fn analyze_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        if self.has_symbol(variant_def.identifier) {
            return Err(Message::multiple_ids(
                variant_def.location,
                variant_def.identifier,
            ));
        }

        for internal in variant_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let variants =
            self.get_variants_from_definition(variant_def.identifier, &variant_def.fields)?;

        self.add_symbol(Entry {
            name: variant_def.identifier,
            is_static: true,
            kind: SymbolKind::from(VariantDefData { variants }),
        });

        Ok(())
    }

    fn get_variants_from_definition(
        &mut self,
        variant_def_name: Ident,
        variants: &BTreeMap<Ident, VariantField>,
    ) -> Result<BTreeMap<Ident, Entry>, Message> {
        let mut res: BTreeMap<Ident, Entry> = BTreeMap::new();

        for (variant_kind_num, (variant_name, variant)) in variants.iter().enumerate() {
            let variant_data = match variant {
                VariantField::Common => VariantData {
                    variant_name: variant_def_name,
                    variant_kind: VariantKind::EnumKind,
                    variant_kind_num,
                },
                VariantField::StructLike(fields) => VariantData {
                    variant_name: variant_def_name,
                    variant_kind: VariantKind::VariantStructKind(VariantStructKind {
                        fields: {
                            let mut variant_fields = BTreeMap::<Ident, StructFieldData>::new();
                            for (field_name, field_ty) in fields.iter() {
                                variant_fields.insert(
                                    *field_name,
                                    StructFieldData {
                                        ty: field_ty.get_type(),
                                    },
                                );
                            }
                            variant_fields
                        },
                    }),
                    variant_kind_num,
                },
                VariantField::TupleLike(fields) => VariantData {
                    variant_name: variant_def_name,
                    variant_kind: VariantKind::VariantTupleKind(VariantTupleKind {
                        fields: {
                            let mut variant_fields = BTreeMap::<usize, StructFieldData>::new();
                            for (field_num, field_ty) in fields.iter().enumerate() {
                                variant_fields.insert(
                                    field_num,
                                    StructFieldData {
                                        ty: field_ty.get_type(),
                                    },
                                );
                            }
                            variant_fields
                        },
                    }),
                    variant_kind_num,
                },
            };

            res.insert(
                *variant_name,
                Entry {
                    name: *variant_name,
                    is_static: true,
                    kind: SymbolKind::Variant(variant_data),
                },
            );
        }

        Ok(res)
    }

    fn access_variant_enum(
        &mut self,
        variant_name: Ident,
        variant_data: &VariantData,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let location = rhs.location();

        let Ast::Value(Value {
            kind: ValueKind::Identifier(_),
            ..
        }) = rhs
        else {
            return Err(Message::no_id_in_namespace(
                location,
                variant_data.variant_name,
                variant_name,
            ));
        };

        Ok(Some(Expression {
            location,
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location,
                    kind: ValueKind::Struct {
                        identifier: variant_data.variant_name,
                        components: vec![
                            (
                                Ident::from("__kind__".to_string()),
                                Ast::Value(Value {
                                    location,
                                    kind: ValueKind::Integer(variant_data.variant_kind_num),
                                }),
                            ),
                            (
                                Ident::from("__data__".to_string()),
                                Ast::Value(Value {
                                    location,
                                    kind: ValueKind::Struct {
                                        identifier: variant_utils::get_variant_data_type_id(
                                            variant_data.variant_name,
                                        ),
                                        components: vec![(
                                            variant_name,
                                            Ast::Value(Value {
                                                location,
                                                kind: ValueKind::Struct {
                                                    identifier: Ident::from(format!(
                                                        "__{}__{}__",
                                                        variant_data.variant_name, variant_name
                                                    )),
                                                    components: vec![],
                                                },
                                            }),
                                        )],
                                    },
                                }),
                            ),
                        ],
                    },
                })),
                ty: Type::Custom(variant_data.variant_name.to_string()),
            },
        }))
    }

    fn access_variant_struct(
        &mut self,
        variant_name: Ident,
        variant_data: &VariantData,
        struct_data: &VariantStructKind,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let location = rhs.location();

        let Ast::Value(Value {
            kind:
                ValueKind::Struct {
                    components: value_comps,
                    ..
                },
            ..
        }) = rhs
        else {
            return Err(Message::no_id_in_namespace(
                location,
                variant_data.variant_name,
                variant_name,
            ));
        };

        self.check_struct_components(value_comps, variant_name, &struct_data.fields)?;

        Ok(Some(Expression {
            location,
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location,
                    kind: ValueKind::Struct {
                        identifier: variant_data.variant_name,
                        components: vec![
                            (
                                Ident::from("__kind__".to_string()),
                                Ast::Value(Value {
                                    location,
                                    kind: ValueKind::Integer(variant_data.variant_kind_num),
                                }),
                            ),
                            (
                                Ident::from("__data__".to_string()),
                                Ast::Value(Value {
                                    location,
                                    kind: ValueKind::Struct {
                                        identifier: variant_utils::get_variant_data_type_id(
                                            variant_data.variant_name,
                                        ),
                                        components: vec![(
                                            variant_name,
                                            Ast::Value(Value {
                                                location,
                                                kind: ValueKind::Struct {
                                                    identifier: Ident::from(format!(
                                                        "__{}__{}__",
                                                        variant_data.variant_name, variant_name
                                                    )),
                                                    components: value_comps.clone(),
                                                },
                                            }),
                                        )],
                                    },
                                }),
                            ),
                        ],
                    },
                })),
                ty: Type::Custom(variant_data.variant_name.to_string()),
            },
        }))
    }

    fn access_variant_tuple(
        &mut self,
        variant_name: Ident,
        variant_data: &VariantData,
        tuple_data: &VariantTupleKind,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        let location = rhs.location();

        let Ast::Value(Value {
            kind:
                ValueKind::Tuple {
                    components: value_comps,
                    ..
                },
            ..
        }) = rhs
        else {
            return Err(Message::no_id_in_namespace(
                location,
                variant_data.variant_name,
                variant_name,
            ));
        };

        self.check_tuple_components(value_comps, Some(variant_name), &tuple_data.fields)?;

        Ok(Some(Expression {
            location,
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location,
                    kind: ValueKind::Struct {
                        identifier: variant_data.variant_name,
                        components: vec![
                            (
                                Ident::from("__kind__".to_string()),
                                Ast::Value(Value {
                                    location,
                                    kind: ValueKind::Integer(variant_data.variant_kind_num),
                                }),
                            ),
                            (
                                Ident::from("__data__".to_string()),
                                Ast::Value(Value {
                                    location,
                                    kind: ValueKind::Struct {
                                        identifier: variant_utils::get_variant_data_type_id(
                                            variant_data.variant_name,
                                        ),
                                        components: vec![(
                                            variant_name,
                                            Ast::Value(Value {
                                                location,
                                                kind: ValueKind::Struct {
                                                    identifier: Ident::from(format!(
                                                        "__{}__{}__",
                                                        variant_data.variant_name, variant_name
                                                    )),
                                                    components: {
                                                        let mut res: Vec<(Ident, Ast)> =
                                                            Vec::with_capacity(value_comps.len());

                                                        for (value_num, value_comp) in
                                                            value_comps.iter().enumerate()
                                                        {
                                                            let field_id = Ident::from(format!(
                                                                "_{value_num}"
                                                            ));
                                                            res.push((
                                                                field_id,
                                                                value_comp.clone(),
                                                            ));
                                                        }

                                                        res
                                                    },
                                                },
                                            }),
                                        )],
                                    },
                                }),
                            ),
                        ],
                    },
                })),
                ty: Type::Custom(variant_data.variant_name.to_string()),
            },
        }))
    }

    pub fn access_variant(
        &mut self,
        variant_name: Ident,
        variant_data: &VariantData,
        rhs: &Ast,
    ) -> Result<Option<Expression>, Message> {
        match &variant_data.variant_kind {
            VariantKind::EnumKind => self.access_variant_enum(variant_name, variant_data, rhs),
            VariantKind::VariantStructKind(struct_data) => {
                self.access_variant_struct(variant_name, variant_data, struct_data, rhs)
            }
            VariantKind::VariantTupleKind(tuple_data) => {
                self.access_variant_tuple(variant_name, variant_data, tuple_data, rhs)
            }
        }
    }
}
