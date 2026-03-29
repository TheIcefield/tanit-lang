use tanitc_hir::hir::expressions::member_access::MemberAccessExpr;

use crate::{symbol_table::type_info::TypeInfo, AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_member_access_expr(&self, expr: &MemberAccessExpr) -> AnalyzeResult<()> {
        Ok(())
    }

    pub(crate) fn get_member_access_expr_type(&self, expr: &MemberAccessExpr) -> TypeInfo {
        TypeInfo::default()
    }

    /*
    pub(crate) fn access_enum(
        &self,
        rhs: &mut Expression,
        enum_data: &EnumData,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        *rhs = Expression::Literal(Literal::Integer(Integer {
            location,
            value: enum_data.value,
        }));

        Ok(())
    }

    pub(crate) fn access_func_def(&mut self, rhs: &mut Expression) -> AnalyzeResult<()> {
        let Expression::Call(call_expr) = rhs else {
            return Err(Message::new(
                rhs.location(),
                format!("Unexpected rhs: {rhs:?}"),
            ));
        };

        self.analyze_call_expr(call_expr)?;

        Ok(())
    }

    pub(crate) fn access_struct_def(
        &mut self,
        node: &mut Expression,
        struct_data: &StructDefData,
    ) -> AnalyzeResult<()> {
        let location = node.location();

        let Expression::Literal(Literal::Struct(StructLiteral {
            name: struct_name,
            fields: value_comps,
            ..
        })) = node
        else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected StructLiteral, actually: {node:?}"),
            ));
        };

        self.check_struct_literal_components(value_comps, struct_data, location)?;

        *struct_name = struct_data.name.clone();

        Ok(())
    }

    pub(crate) fn access_union_def(
        &mut self,
        node: &mut Expression,
        union_data: &UnionDefData,
    ) -> AnalyzeResult<()> {
        let Expression::Literal(Literal::Struct(StructLiteral {
            location,
            name: union_name,
            fields: value_comps,
        })) = node
        else {
            return Err(Message::unreachable(
                node.location(),
                format!("expected StructLiteral, actually: {}", node.kind_str()),
            ));
        };

        self.check_union_literal_components(value_comps, union_data, *location)?;

        *union_name = union_data.name.clone();

        Ok(())
    }


    fn access_variant_enum(
        &mut self,
        rhs: &mut Expression,
        variant_data: &VariantData,
        variant_kind_name: NameSpec,
        variant_data_name: NameSpec,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        if !matches!(rhs, Expression::Variable(_)) {
            return Err(Message::no_id_in_namespace(
                location,
                &variant_data.variant_name,
                variant_data.variant_unit_id,
            ));
        }

        *rhs = Expression::Literal(Literal::Struct(StructLiteral {
            location,
            name: variant_data.variant_name.clone(),
            fields: vec![
                (
                    Ident::from("__kind__".to_string()),
                    Expression::Variable(Variable {
                        location,
                        name: variant_kind_name,
                    }),
                ),
                (
                    Ident::from("__data__".to_string()),
                    Expression::Literal(Literal::Struct(StructLiteral {
                        location,
                        name: VariantDef::get_variant_data_type_name(&variant_data.variant_name),
                        fields: vec![(
                            Ident::from(variant_data.variant_unit_id),
                            Expression::Literal(Literal::Struct(StructLiteral {
                                location,
                                name: variant_data_name,
                                fields: vec![],
                            })),
                        )],
                    })),
                ),
            ],
        }));

        Ok(())
    }

    fn access_variant_struct(
        &mut self,
        rhs: &mut Expression,
        variant_data: &VariantData,
        variant_struct_data: &VariantStruct,
        variant_kind_name: NameSpec,
        variant_data_name: NameSpec,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        let Expression::Literal(Literal::Struct(StructLiteral {
            fields: variant_struct_fields,
            ..
        })) = rhs
        else {
            return Err(Message::no_id_in_namespace(
                location,
                &variant_data.variant_name,
                variant_data.variant_unit_id,
            ));
        };

        let mut variant_struct_fields = std::mem::take(variant_struct_fields);

        self.check_variant_struct_components(
            &mut variant_struct_fields,
            variant_struct_data,
            location,
        )?;

        *rhs = Expression::Literal(Literal::Struct(StructLiteral {
            location,
            name: variant_data.variant_name.clone(),
            fields: vec![
                (
                    Ident::from("__kind__".to_string()),
                    Expression::Variable(Variable {
                        location,
                        name: variant_kind_name,
                    }),
                ),
                (
                    Ident::from("__data__".to_string()),
                    Expression::Literal(Literal::Struct(StructLiteral {
                        location,
                        name: VariantDef::get_variant_data_type_name(&variant_data.variant_name),
                        fields: vec![(
                            Ident::from(variant_data.variant_unit_id),
                            Expression::Literal(Literal::Struct(StructLiteral {
                                location,
                                name: variant_data_name,
                                fields: variant_struct_fields,
                            })),
                        )],
                    })),
                ),
            ],
        }));

        Ok(())
    }

    fn access_variant_tuple(
        &self,
        rhs: &mut Expression,
        variant_data: &VariantData,
        tuple_data: &VariantTuple,
        variant_kind_name: NameSpec,
        variant_data_name: NameSpec,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();
        let variant_id = variant_data
            .variant_name
            .get_id()
            .ok_or(Message::empty_name_spec(location))?;

        let Expression::Literal(Literal::Tuple(TupleLiteral {
            units: variant_tuple_units,
            ..
        })) = rhs
        else {
            return Err(Message::no_id_in_namespace(
                location,
                &variant_data.variant_name,
                variant_data.variant_unit_id,
            ));
        };

        let variant_tuple_units = std::mem::take(variant_tuple_units);
        self.check_tuple_components(&variant_tuple_units, &tuple_data.fields, location)?;

        let mut variant_data_fields =
            Vec::<(Ident, Expression)>::with_capacity(variant_tuple_units.len());

        for (value_num, value_comp) in variant_tuple_units.into_iter().enumerate() {
            let field_id = Ident::from(format!("_{value_num}"));
            variant_data_fields.push((field_id, value_comp));
        }

        *rhs = Expression::Literal(Literal::Struct(StructLiteral {
            location,
            name: variant_data.variant_name.clone(),
            fields: vec![
                (
                    Ident::from("__kind__".to_string()),
                    Expression::Variable(Variable {
                        location,
                        name: variant_kind_name,
                    }),
                ),
                (
                    Ident::from("__data__".to_string()),
                    Expression::Literal(Literal::Struct(StructLiteral {
                        location,
                        name: VariantDef::get_variant_data_type_name(&variant_data.variant_name),
                        fields: vec![(
                            variant_id,
                            Expression::Literal(Literal::Struct(StructLiteral {
                                location,
                                name: variant_data_name,
                                fields: variant_data_fields,
                            })),
                        )],
                    })),
                ),
            ],
        }));

        Ok(())
    }

    pub(crate) fn access_variant(
        &mut self,
        rhs: &mut Expression,
        variant_data: &VariantData,
    ) -> AnalyzeResult<()> {
        let variant_kind_name = VariantDef::get_variant_kind_name(
            &variant_data.variant_name,
            variant_data.variant_unit_id,
        );

        let variant_data_name = VariantDef::get_variant_data_name(
            &variant_data.variant_name,
            variant_data.variant_unit_id,
        );

        match &variant_data.variant_kind {
            VariantKind::Enum => {
                self.access_variant_enum(rhs, variant_data, variant_kind_name, variant_data_name)
            }
            VariantKind::Struct(variant_struct_data) => self.access_variant_struct(
                rhs,
                variant_data,
                variant_struct_data,
                variant_kind_name,
                variant_data_name,
            ),
            VariantKind::Tuple(variant_tuple_data) => self.access_variant_tuple(
                rhs,
                variant_data,
                variant_tuple_data,
                variant_kind_name,
                variant_data_name,
            ),
        }
    }
    */
}
