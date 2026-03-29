use std::collections::BTreeMap;

use tanitc_attributes::Mutability;
use tanitc_hir::hir::{
    expressions::{
        literal::{ArrayLiteral, Literal, StructLiteral, TupleLiteral},
        Expression,
    },
    type_spec::{ArraySize, RefType, TupleType, Type},
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{
    hir::expressions::get_ordinal_number_suffix,
    symbol_table::{
        entry::{
            StructDefData, StructFieldData, StructFieldsData, SymbolKind, UnionDefData,
            VariantStruct,
        },
        type_info::{MemberInfo, TypeInfo},
    },
    AnalyzeResult, Analyzer,
};

impl Analyzer {
    pub(crate) fn analyze_literal(&mut self, lit: &mut Literal) -> AnalyzeResult<()> {
        match lit {
            Literal::Integer(_) => Ok(()),
            Literal::Decimal(_) => Ok(()),
            Literal::Text(_) => Ok(()),
            Literal::Struct(lit) => self.analyze_struct_literal(lit),
            Literal::Tuple(lit) => self.analyze_tuple_literal(lit),
            Literal::Array(lit) => self.analyze_array_literal(lit),
        }
    }

    pub(crate) fn get_literal_type(&self, lit: &Literal) -> TypeInfo {
        match lit {
            Literal::Text(_) => TypeInfo {
                ty: Type::Ref(RefType {
                    ref_to: Box::new(Type::Str),
                    mutability: Mutability::Immutable,
                }),
                mutability: Mutability::Immutable,
                ..Default::default()
            },
            Literal::Decimal(_) => TypeInfo {
                ty: Type::F32,
                mutability: Mutability::Mutable,
                ..Default::default()
            },
            Literal::Integer(_) => TypeInfo {
                ty: Type::I32,
                mutability: Mutability::Mutable,
                ..Default::default()
            },

            Literal::Struct(StructLiteral { name, .. }) => {
                let ty = Type::Custom(name.clone());
                let Some(mut type_info) = self.table.lookup_type(&ty) else {
                    return TypeInfo {
                        ty,
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };
                type_info.mutability = Mutability::Mutable;
                type_info
            }
            Literal::Tuple(TupleLiteral { units, .. }) => {
                let mut comp_vec = Vec::<Type>::new();
                for comp in units.iter() {
                    comp_vec.push(self.get_expr_type(comp).ty);
                }
                TypeInfo {
                    ty: Type::Tuple(TupleType {
                        units: comp_vec.clone(),
                    }),
                    mutability: Mutability::Mutable,
                    members: {
                        let mut members = BTreeMap::<Ident, MemberInfo>::new();
                        for (comp_idx, comp_type) in comp_vec.iter().enumerate() {
                            members.insert(
                                Ident::from(format!("{comp_idx}")),
                                MemberInfo {
                                    is_public: true,
                                    ty: comp_type.clone(),
                                },
                            );
                        }
                        members
                    },
                    ..Default::default()
                }
            }
            Literal::Array(ArrayLiteral { elements, .. }) => {
                let len = elements.len();
                if len == 0 {
                    return TypeInfo {
                        ty: Type::Array {
                            size: ArraySize::Unknown,
                            value_type: Box::new(Type::Auto),
                        },
                        mutability: Mutability::Mutable,
                        members: BTreeMap::new(),
                        ..Default::default()
                    };
                }

                TypeInfo {
                    ty: Type::Array {
                        size: ArraySize::Fixed(len),
                        value_type: Box::new(self.get_expr_type(&elements[0]).ty),
                    },
                    mutability: Mutability::Mutable,
                    ..Default::default()
                }
            }
        }
    }

    pub(crate) fn check_struct_literal_components(
        &mut self,
        value_comps: &mut [(Ident, Expression)],
        struct_data: &StructDefData,
        location: Location,
    ) -> AnalyzeResult<()> {
        let struct_comps = &struct_data.fields;
        if value_comps.len() != struct_comps.len() {
            return Err(Message::new(
                location,
                format!(
                    "Struct \"{}\" consists of {} fields, but {} were supplied",
                    struct_data.name,
                    struct_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            if let Err(mut msg) =
                self.check_struct_component(comp_id, value_comps, &struct_data.fields)
            {
                msg.text = format!("Struct {}", msg.text);
                self.error(msg);
            }
        }

        Ok(())
    }

    pub(crate) fn check_union_literal_components(
        &mut self,
        value_comps: &mut [(Ident, Expression)],
        union_data: &UnionDefData,
        location: Location,
    ) -> AnalyzeResult<()> {
        let union_comp_size = union_data.fields.len();
        let initialized_comp_size = value_comps.len();

        if union_comp_size == 0 && initialized_comp_size > 0 {
            return Err(Message::new(
                location,
                format!(
                    "Union \"{}\" has no fields, but were supplied {initialized_comp_size} fields",
                    union_data.name
                ),
            ));
        }

        if union_comp_size > 0 && initialized_comp_size > 1 {
            return Err(Message::new(
                location,
                format!(
                    "Only one union field must be initialized, but {initialized_comp_size} were initialized",
                ),
            ));
        }

        for comp_id in 0..initialized_comp_size {
            if let Err(mut msg) =
                self.check_struct_component(comp_id, value_comps, &union_data.fields)
            {
                msg.text = format!("Union {}", msg.text);
                self.error(msg);
            }
        }

        Ok(())
    }

    pub(crate) fn check_variant_struct_components(
        &mut self,
        value_comps: &mut [(Ident, Expression)],
        variant_struct_data: &VariantStruct,
        location: Location,
    ) -> AnalyzeResult<()> {
        let struct_comps = &variant_struct_data.fields;
        if value_comps.len() != struct_comps.len() {
            return Err(Message::new(
                location,
                format!(
                    "Variant struct \"{}\" consists of {} fields, but {} were supplied",
                    variant_struct_data.name,
                    struct_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            if let Err(mut msg) =
                self.check_struct_component(comp_id, value_comps, &variant_struct_data.fields)
            {
                msg.text = format!("Variant {}", msg.text);
                self.error(msg);
            }
        }

        Ok(())
    }

    pub(crate) fn check_tuple_components(
        &self,
        value_comps: &[Expression],
        tuple_comps: &BTreeMap<usize, StructFieldData>,
        location: Location,
    ) -> AnalyzeResult<()> {
        if value_comps.len() != tuple_comps.len() {
            return Err(Message::new(
                location,
                format!(
                    "Tuple consists of {} fields, but {} were supplied",
                    tuple_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            let Some(value_comp) = value_comps.get(comp_id) else {
                continue;
            };
            let Some(tuple_comp_type) = tuple_comps.get(&comp_id) else {
                continue;
            };

            let value_comp_type = self.get_expr_type(value_comp);

            let mut alias_to = self.find_alias_value(&tuple_comp_type.ty);

            if value_comp_type.ty == tuple_comp_type.ty {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type.ty != tuple_comp_type.ty {
                return Err(Message::new(
                    value_comp.location(),
                    format!(
                        "Tuple component with index \"{comp_id}\" is {}, but initialized like {}",
                        tuple_comp_type.ty, value_comp_type.ty
                    ),
                ));
            }

            if let Some(alias_to_ty) = &alias_to {
                if value_comp_type.ty != *alias_to_ty {
                    return Err(Message::new(
                        value_comp.location(),
                        format!(
                        "Tuple component with index \"{comp_id}\" is {} (aka: {}), but initialized like {value_comp_type}",
                        tuple_comp_type.ty,
                        alias_to_ty
                    )));
                }
            }
        }

        Ok(())
    }

    fn analyze_struct_literal(&mut self, literal: &mut StructLiteral) -> AnalyzeResult<()> {
        let mut entry = self
            .table
            .lookup_name_spec(&literal.name)
            .map_err(|err| Message::new(literal.location, err))?
            .clone();

        if let SymbolKind::AliasDef(alias_data) = &entry.kind {
            let ty = &alias_data.ty;
            match ty {
                Type::Custom(alias_to_name) => {
                    entry = self
                        .table
                        .lookup_name_spec(alias_to_name)
                        .map_err(|err| Message::new(literal.location, err))?
                        .clone();
                }
                ty if ty.is_common() => {
                    return Err(Message::new(
                        literal.location,
                        format!("Common type \"{ty}\" does not have any fields"),
                    ))
                }
                _ => {
                    todo!("Unexpected type: {ty}");
                }
            }
        }

        match &entry.kind {
            SymbolKind::StructDef(struct_data) => {
                self.check_struct_literal_components(
                    &mut literal.fields,
                    struct_data,
                    literal.location,
                )?;
            }
            SymbolKind::UnionDef(union_data) => {
                self.check_union_literal_components(
                    &mut literal.fields,
                    union_data,
                    literal.location,
                )?;
            }
            _ => {
                return Err(Message::new(
                    literal.location,
                    format!(
                        "Cannot find struct or union named \"{}\" in this scope",
                        literal.name
                    ),
                ));
            }
        }

        Ok(())
    }

    fn analyze_tuple_literal(&mut self, literal: &mut TupleLiteral) -> AnalyzeResult<()> {
        for unit in literal.units.iter_mut() {
            if let Err(err) = self.analyze_expression(unit) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn analyze_array_literal(&mut self, literal: &mut ArrayLiteral) -> AnalyzeResult<()> {
        if literal.elements.is_empty() {
            return Ok(());
        }

        let comp_type = self.get_expr_type(&literal.elements[0]);

        for comp in literal.elements.iter().enumerate() {
            let current_comp_type = self.get_expr_type(comp.1);
            if comp_type.ty != current_comp_type.ty {
                let comp_index = comp.0 + 1;
                let suffix = get_ordinal_number_suffix(comp.0);
                return Err(Message::new(
                    comp.1.location(),
                    format!(
                        "Array type is declared like {}, but {comp_index}{suffix} element has type {}",
                        comp_type.ty, current_comp_type.ty
                    ),
                ));
            }
        }

        Ok(())
    }

    fn check_struct_component(
        &mut self,
        comp_id: usize,
        value_comps: &mut [(Ident, Expression)],
        struct_fields: &StructFieldsData,
    ) -> AnalyzeResult<()> {
        let value_comp = value_comps.get_mut(comp_id).unwrap();
        let value_comp_name = &value_comp.0;
        let value_comp_type = self.get_expr_type(&value_comp.1);
        let struct_comp_type = &struct_fields.get(value_comp_name).unwrap().ty;

        if let Err(err) = self.analyze_expression(&mut value_comp.1) {
            self.error(err);
        }

        if self
            .compare_types(
                struct_comp_type,
                &value_comp_type.ty,
                value_comp.1.location(),
            )
            .is_err()
        {
            return Err(Message::new(
                value_comp.1.location(),
                format!("field named \"{value_comp_name}\" is {struct_comp_type}, but initialized like {value_comp_type}"),
            ));
        }

        Ok(())
    }
}
