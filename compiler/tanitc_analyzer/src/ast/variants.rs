use std::collections::BTreeMap;

use tanitc_ast::ast::{
    expressions::{Expression, ExpressionKind},
    structs::StructFieldInfo,
    types::TypeSpec,
    values::{Value, ValueKind},
    variants::{get_variant_data_type_id, VariantDef, VariantField},
    Ast,
};
use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{
    Entry, StructFieldData, SymbolKind, VariantData, VariantDefData, VariantKind,
    VariantStructKind, VariantTupleKind,
};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        self.check_variants_are_allowed(&variant_def.location)?;

        if self.has_symbol(variant_def.name.id) {
            return Err(Message::multiple_ids(
                &variant_def.location,
                variant_def.name.id,
            ));
        }

        variant_def.name.prefix = self.table.get_id();

        for internal in variant_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        let variants = self.get_variants_from_definition(variant_def)?;

        self.add_symbol(Entry {
            name: variant_def.name.id,
            is_static: true,
            kind: SymbolKind::from(VariantDefData {
                name: variant_def.name,
                variants,
            }),
        });

        Ok(())
    }

    fn check_variants_are_allowed(&self, location: &Location) -> Result<(), Message> {
        if !self.compile_options.allow_variants {
            return Err(Message::new(
                location,
                "Variants not supported in 0.1.0 (use \"--variants\" to enable variants)",
            ));
        }

        Ok(())
    }

    fn get_enum_variant_from_def(&self) -> VariantData {
        VariantData {
            variant_name: Name::default(),
            variant_unit_name: Ident::default(),
            variant_kind: VariantKind::EnumKind,
            variant_kind_num: 0,
        }
    }

    fn get_struct_variant_from_def(
        &self,
        variant_name: Name,
        variant_struct_fields: &BTreeMap<Ident, StructFieldInfo>,
    ) -> VariantData {
        VariantData {
            variant_name,
            variant_unit_name: Ident::default(),
            variant_kind: VariantKind::VariantStructKind(VariantStructKind {
                variant_name,
                fields: {
                    let mut variant_fields = BTreeMap::<Ident, StructFieldData>::new();
                    for (field_name, field_ty) in variant_struct_fields.iter() {
                        variant_fields.insert(
                            *field_name,
                            StructFieldData {
                                struct_name: Name::default(),
                                ty: field_ty.ty.get_type(),
                            },
                        );
                    }
                    variant_fields
                },
            }),
            variant_kind_num: 0,
        }
    }

    fn get_tuple_variant_from_def(&self, variant_tuple_components: &[TypeSpec]) -> VariantData {
        VariantData {
            variant_name: Name::default(),
            variant_unit_name: Ident::default(),
            variant_kind: VariantKind::VariantTupleKind(VariantTupleKind {
                fields: {
                    let mut variant_fields = BTreeMap::<usize, StructFieldData>::new();
                    for (field_num, field_ty) in variant_tuple_components.iter().enumerate() {
                        variant_fields.insert(
                            field_num,
                            StructFieldData {
                                struct_name: Name::default(),
                                ty: field_ty.get_type(),
                            },
                        );
                    }
                    variant_fields
                },
            }),
            variant_kind_num: 0,
        }
    }

    fn get_variants_from_definition(
        &self,
        variant_def: &VariantDef,
    ) -> Result<BTreeMap<Ident, Entry>, Message> {
        let mut res: BTreeMap<Ident, Entry> = BTreeMap::new();

        for (variant_kind_num, (variant_unit_name, variant)) in
            variant_def.fields.iter().enumerate()
        {
            let name = variant_def.name;
            let mut variant_data = match variant {
                VariantField::Common => self.get_enum_variant_from_def(),
                VariantField::StructLike(fields) => self.get_struct_variant_from_def(name, fields),
                VariantField::TupleLike(fields) => self.get_tuple_variant_from_def(fields),
            };

            variant_data.variant_name = variant_def.name;
            variant_data.variant_kind_num = variant_kind_num;
            variant_data.variant_unit_name = *variant_unit_name;

            res.insert(
                *variant_unit_name,
                Entry {
                    name: *variant_unit_name,
                    is_static: true,
                    kind: SymbolKind::Variant(variant_data),
                },
            );
        }

        Ok(res)
    }

    fn access_variant_enum(
        &mut self,
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
                &location,
                variant_data.variant_name.id,
                variant_data.variant_unit_name,
            ));
        };

        Ok(Some(Expression {
            location: location.clone(),
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location: location.clone(),
                    kind: ValueKind::Struct {
                        name: variant_data.variant_name,
                        components: vec![
                            (
                                Name::from("__kind__".to_string()),
                                Ast::Value(Value {
                                    location: location.clone(),
                                    kind: ValueKind::Identifier(Ident::from(format!(
                                        "__{}__kind__{}__",
                                        variant_data.variant_name, variant_data.variant_unit_name
                                    ))),
                                }),
                            ),
                            (
                                Name::from("__data__".to_string()),
                                Ast::Value(Value {
                                    location: location.clone(),
                                    kind: ValueKind::Struct {
                                        name: get_variant_data_type_id(variant_data.variant_name),
                                        components: vec![(
                                            Name::from(variant_data.variant_unit_name),
                                            Ast::Value(Value {
                                                location,
                                                kind: ValueKind::Struct {
                                                    name: Name::from(format!(
                                                        "__{}__data__{}__",
                                                        variant_data.variant_name,
                                                        variant_data.variant_unit_name
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
                ty: Type::Custom(variant_data.variant_name),
            },
        }))
    }

    fn access_variant_struct(
        &mut self,
        variant_data: &VariantData,
        variant_struct_data: &VariantStructKind,
        rhs: &mut Ast,
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
                &location,
                variant_data.variant_name.id,
                variant_data.variant_unit_name,
            ));
        };

        self.check_variant_struct_components(value_comps, variant_struct_data, &location)?;

        Ok(Some(Expression {
            location: location.clone(),
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location: location.clone(),
                    kind: ValueKind::Struct {
                        name: variant_data.variant_name,
                        components: vec![
                            (
                                Name::from("__kind__".to_string()),
                                Ast::Value(Value {
                                    location: location.clone(),
                                    kind: ValueKind::Identifier(Ident::from(format!(
                                        "__{}__kind__{}__",
                                        variant_data.variant_name, variant_data.variant_unit_name
                                    ))),
                                }),
                            ),
                            (
                                Name::from("__data__".to_string()),
                                Ast::Value(Value {
                                    location: location.clone(),
                                    kind: ValueKind::Struct {
                                        name: get_variant_data_type_id(variant_data.variant_name),
                                        components: vec![(
                                            Name::from(variant_data.variant_unit_name),
                                            Ast::Value(Value {
                                                location,
                                                kind: ValueKind::Struct {
                                                    name: Name::from(format!(
                                                        "__{}__data__{}__",
                                                        variant_data.variant_name,
                                                        variant_data.variant_unit_name
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
                ty: Type::Custom(variant_data.variant_name),
            },
        }))
    }

    fn access_variant_tuple(
        &mut self,
        variant_name: Name,
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
                &location,
                variant_data.variant_name.id,
                variant_name.id,
            ));
        };

        self.check_tuple_components(
            value_comps,
            Some(variant_name.id),
            &tuple_data.fields,
            &location,
        )?;

        Ok(Some(Expression {
            location: location.clone(),
            kind: ExpressionKind::Term {
                node: Box::new(Ast::Value(Value {
                    location: location.clone(),
                    kind: ValueKind::Struct {
                        name: variant_data.variant_name,
                        components: vec![
                            (
                                Name::from("__kind__".to_string()),
                                Ast::Value(Value {
                                    location: location.clone(),
                                    kind: ValueKind::Identifier(Ident::from(format!(
                                        "__{}__kind__{}__",
                                        variant_data.variant_name, variant_name
                                    ))),
                                }),
                            ),
                            (
                                Name::from("__data__".to_string()),
                                Ast::Value(Value {
                                    location: location.clone(),
                                    kind: ValueKind::Struct {
                                        name: get_variant_data_type_id(variant_data.variant_name),
                                        components: vec![(
                                            variant_name,
                                            Ast::Value(Value {
                                                location,
                                                kind: ValueKind::Struct {
                                                    name: Name::from(format!(
                                                        "__{}__data__{}__",
                                                        variant_data.variant_name, variant_name
                                                    )),
                                                    components: {
                                                        let mut res: Vec<(Name, Ast)> =
                                                            Vec::with_capacity(value_comps.len());

                                                        for (value_num, value_comp) in
                                                            value_comps.iter().enumerate()
                                                        {
                                                            let field_id =
                                                                Name::from(format!("_{value_num}"));
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
                ty: Type::Custom(variant_data.variant_name),
            },
        }))
    }

    pub fn access_variant(
        &mut self,
        variant_data: &VariantData,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        match &variant_data.variant_kind {
            VariantKind::EnumKind => self.access_variant_enum(variant_data, rhs),
            VariantKind::VariantStructKind(struct_data) => {
                self.access_variant_struct(variant_data, struct_data, rhs)
            }
            VariantKind::VariantTupleKind(tuple_data) => {
                self.access_variant_tuple(variant_data.variant_name, variant_data, tuple_data, rhs)
            }
        }
    }
}
