use std::collections::BTreeMap;

use crate::symbol_table::entry::{
    Entry, StructFieldData, StructFieldsData, SymbolKind, VariantData, VariantDefData, VariantKind,
    VariantStruct, VariantTuple,
};
use tanitc_hir::hir::{
    definitions::{
        structs::StructFieldsInfo,
        variants::{get_variant_data_type_id, VariantDef, VariantField},
    },
    expressions::{
        literal::{Literal, StructLiteral, TupleLiteral},
        variable::Variable,
        Expression,
    },
    types::Type,
};
use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_variant_def(
        &mut self,
        variant_def: &mut VariantDef,
    ) -> AnalyzeResult<()> {
        self.check_variants_are_allowed(variant_def.location)?;

        if self.has_symbol(variant_def.name.id) {
            return Err(Message::multiple_ids(
                variant_def.location,
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

    fn check_variants_are_allowed(&self, location: Location) -> AnalyzeResult<()> {
        if !self.compile_options.allow_variants {
            return Err(Message::new(
                location,
                "Variants are not supported in 0.1.0 (use \"--variants\" to enable variants)",
            ));
        }

        Ok(())
    }

    fn get_enum_variant_from_def(&self) -> VariantData {
        VariantData {
            variant_name: Name::default(),
            variant_unit_name: Ident::default(),
            variant_kind: VariantKind::Enum,
            variant_kind_num: 0,
        }
    }

    fn get_struct_variant_from_def(
        &self,
        variant_name: Name,
        variant_struct_fields: &StructFieldsInfo,
    ) -> VariantData {
        VariantData {
            variant_name,
            variant_unit_name: Ident::default(),
            variant_kind: VariantKind::Struct(VariantStruct {
                variant_name,
                fields: {
                    let mut variant_fields = StructFieldsData::new();
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

    fn get_tuple_variant_from_def(&self, variant_tuple_components: &[Type]) -> VariantData {
        VariantData {
            variant_name: Name::default(),
            variant_unit_name: Ident::default(),
            variant_kind: VariantKind::Tuple(VariantTuple {
                fields: {
                    let mut variant_fields = BTreeMap::<usize, StructFieldData>::new();
                    for (field_num, field_ty) in variant_tuple_components.iter().enumerate() {
                        variant_fields.insert(
                            field_num,
                            StructFieldData {
                                struct_name: Name::default(),
                                ty: field_ty.clone(),
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
                VariantField::Enum => self.get_enum_variant_from_def(),
                VariantField::Struct(fields) => self.get_struct_variant_from_def(name, fields),
                VariantField::Tuple(fields) => self.get_tuple_variant_from_def(fields),
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
        rhs: &mut Expression,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        if !matches!(rhs, Expression::Variable(_)) {
            return Err(Message::no_id_in_namespace(
                location,
                variant_data.variant_name.id,
                variant_data.variant_unit_name,
            ));
        }

        *rhs = Expression::Literal(Literal::Struct(StructLiteral {
            location,
            id: variant_data.variant_name,
            fields: vec![
                (
                    Name::from("__kind__".to_string()),
                    Expression::Variable(Variable {
                        location,
                        id: Ident::from(format!(
                            "__{}__kind__{}__",
                            variant_data.variant_name, variant_data.variant_unit_name
                        )),
                    }),
                ),
                (
                    Name::from("__data__".to_string()),
                    Expression::Literal(Literal::Struct(StructLiteral {
                        location,
                        id: get_variant_data_type_id(variant_data.variant_name),
                        fields: vec![(
                            Name::from(variant_data.variant_unit_name),
                            Expression::Literal(Literal::Struct(StructLiteral {
                                location,
                                id: Name::from(format!(
                                    "__{}__data__{}__",
                                    variant_data.variant_name, variant_data.variant_unit_name
                                )),
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
        variant_data: &VariantData,
        variant_struct_data: &VariantStruct,
        rhs: &mut Expression,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        let Expression::Literal(Literal::Struct(StructLiteral { fields, .. })) = rhs else {
            return Err(Message::no_id_in_namespace(
                location,
                variant_data.variant_name.id,
                variant_data.variant_unit_name,
            ));
        };

        self.check_variant_struct_components(fields, variant_struct_data, location)?;

        let fields = fields.clone();

        *rhs = Expression::Literal(Literal::Struct(StructLiteral {
            location,
            id: variant_data.variant_name,
            fields: vec![
                (
                    Name::from("__kind__".to_string()),
                    Expression::Variable(Variable {
                        location,
                        id: Ident::from(format!(
                            "__{}__kind__{}__",
                            variant_data.variant_name, variant_data.variant_unit_name
                        )),
                    }),
                ),
                (
                    Name::from("__data__".to_string()),
                    Expression::Literal(Literal::Struct(StructLiteral {
                        location,
                        id: get_variant_data_type_id(variant_data.variant_name),
                        fields: vec![(
                            Name::from(variant_data.variant_unit_name),
                            Expression::Literal(Literal::Struct(StructLiteral {
                                location,
                                id: Name::from(format!(
                                    "__{}__data__{}__",
                                    variant_data.variant_name, variant_data.variant_unit_name
                                )),
                                fields,
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
        variant_data: &VariantData,
        tuple_data: &VariantTuple,
        rhs: &mut Expression,
    ) -> AnalyzeResult<()> {
        let location = rhs.location();

        let Expression::Literal(Literal::Tuple(TupleLiteral { units, .. })) = rhs else {
            return Err(Message::no_id_in_namespace(
                location,
                variant_data.variant_name.id,
                variant_data.variant_unit_name,
            ));
        };

        self.check_tuple_components(
            units,
            Some(variant_data.variant_name.id),
            &tuple_data.fields,
            location,
        )?;

        *rhs = Expression::Literal(Literal::Struct(StructLiteral {
            location,
            id: variant_data.variant_name,
            fields: vec![
                (
                    Name::from("__kind__".to_string()),
                    Expression::Variable(Variable {
                        location,
                        id: Ident::from(format!(
                            "__{}__kind__{}__",
                            variant_data.variant_name, variant_data.variant_name
                        )),
                    }),
                ),
                (
                    Name::from("__data__".to_string()),
                    Expression::Literal(Literal::Struct(StructLiteral {
                        location,
                        id: get_variant_data_type_id(variant_data.variant_name),
                        fields: vec![(
                            variant_data.variant_name,
                            Expression::Literal(Literal::Struct(StructLiteral {
                                location,
                                id: Name::from(format!(
                                    "__{}__data__{}__",
                                    variant_data.variant_name, variant_data.variant_name
                                )),
                                fields: {
                                    let mut res: Vec<(Name, Expression)> =
                                        Vec::with_capacity(units.len());

                                    for (value_num, value_comp) in units.iter().enumerate() {
                                        let field_id = Name::from(format!("_{value_num}"));
                                        res.push((field_id, value_comp.clone()));
                                    }

                                    res
                                },
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
        variant_data: &VariantData,
        rhs: &mut Expression,
    ) -> AnalyzeResult<()> {
        match &variant_data.variant_kind {
            VariantKind::Enum => self.access_variant_enum(variant_data, rhs),
            VariantKind::Struct(data) => self.access_variant_struct(variant_data, data, rhs),
            VariantKind::Tuple(data) => self.access_variant_tuple(variant_data, data, rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use tanitc_hir::hir::{blocks::Block, Hir};
    use tanitc_hir_test::{
        create_block, create_enum_variantfield, create_main_func_def, create_struct_variantfield,
        create_tuple_variantfield, create_variant_def,
    };
    use tanitc_options::CompileOptions;

    use super::*;

    fn create_full_variant() -> VariantDef {
        create_variant_def(
            "MyVariant",
            vec![
                create_enum_variantfield("Enum"),
                create_struct_variantfield("Struct", vec![("f1", Type::I32), ("f2", Type::F32)]),
                create_tuple_variantfield("Tuple", vec![Type::I32, Type::F32]),
            ],
        )
    }

    #[test]
    fn variant_use_without_flag_bad_test() {
        // Given
        let variant_def = create_full_variant();
        let main_func = create_main_func_def(vec![]);

        let mut program = Hir::from(Block {
            is_global: true,
            statements: vec![variant_def.into(), main_func.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED_ERR: &str = "Semantic error: Variants are not supported in 0.1.0 (use \"--variants\" to enable variants)";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn variant_use_with_flag_good_test() {
        // Given
        let variant_def = create_full_variant();
        let main_func = create_main_func_def(vec![]);

        let mut program = Hir::from(create_block(vec![variant_def.into(), main_func.into()]));

        let mut analyzer = Analyzer::with_compile_options(CompileOptions {
            allow_variants: true,
            ..Default::default()
        });

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        res.expect("Expected no errors");
    }

    /*
        #[test]
        fn variant_work_test() {
            const SRC_TEXT: &str = "\npub variant MyVariant\
                                \n{\
                                \n    f1\
                                \n    f2(i32, i32)\
                                \n    f3 {\
                                \n        x: i32\
                                \n        y: f32\
                                \n    }\
                                \n}\
                                \nfunc main() {\
                                \n    var v1 = MyVariant::f1\
                                \n    var v3 = MyVariant::f3 {\
                                \n                 x: 4,\
                                \n                 y: 7.5\
                                \n             }\
                                \n}";

            let mut parser = Parser::from_text(SRC_TEXT);

            let ast = parser.parse_program().unwrap();

            let mut hir = {
                let mut lowering = AstLowering::new();
                lowering.low(ast.as_ref()).unwrap()
            };

            {
                let compile_options = CompileOptions {
                    allow_variants: true,
                    ..Default::default()
                };

                let mut analyzer = Analyzer::with_compile_options(compile_options);

                analyzer.analyze_program(hir.as_mut()).unwrap();
            }

        }

        #[test]
        fn variant_in_module_work_test() {
            const SRC_TEXT: &str = "\nmodule math {\
                                \n    variant MyVariant\
                                \n    {\
                                \n        f1\
                                \n        f2(i32, i32)\
                                \n        f3 {\
                                \n            x: i32\
                                \n            y: f32\
                                \n        }\
                                \n    }\
                                \n}\
                                \nfunc main() {\
                                \n    var v1 = math::MyVariant::f1\
                                \n    var v3 = math::MyVariant::f3 {\
                                \n                 x: 4,\
                                \n                 y: 7.5\
                                \n             }\
                                \n}";

            let mut parser = Parser::from_text(SRC_TEXT);

            let ast = parser.parse_program().unwrap();

            let mut hir = {
                let mut lowering = AstLowering::new();
                lowering.low(ast.as_ref()).unwrap()
            };

            {
                let compile_options = CompileOptions {
                    allow_variants: true,
                    ..Default::default()
                };

                let mut analyzer = Analyzer::with_compile_options(compile_options);
                analyzer.analyze_program(hir.as_mut()).unwrap();
            }

            {
                const HEADER_EXPECTED: &str = "typedef enum {\
                                         \n    __math__MyVariant__kind__f1__,\
                                         \n    __math__MyVariant__kind__f2__,\
                                         \n    __math__MyVariant__kind__f3__,\
                                         \n} __math__MyVariant__kind__;\
                                         \n\
                                         \ntypedef struct { } __math__MyVariant__data__f1__;\
                                         \n\
                                         \ntypedef struct {\
                                         \n    signed int _0;\
                                         \n    signed int _1;\
                                         \n} __math__MyVariant__data__f2__;\
                                         \n\
                                         \ntypedef struct {\
                                         \n    signed int x;\
                                         \n    float y;\
                                         \n} __math__MyVariant__data__f3__;\
                                         \n\
                                         \ntypedef union {\
                                         \n    __math__MyVariant__data__f1__ f1;\
                                         \n    __math__MyVariant__data__f2__ f2;\
                                         \n    __math__MyVariant__data__f3__ f3;\
                                         \n} __math__MyVariant__data__;\
                                         \n\
                                         \ntypedef struct {\
                                         \n    __math__MyVariant__kind__ __kind__;\
                                         \n    __math__MyVariant__data__ __data__;\
                                         \n} math__MyVariant;\
                                         \n\
                                         \nvoid main();\n";

                const SOURCE_EXPECTED: &str = "void main()\
                                         \n{\
                                         \n    math__MyVariant const v1 = (math__MyVariant)\
                                         \n    {\
                                         \n        .__kind__=__math__MyVariant__kind__f1__,\
                                         \n        .__data__=(__math__MyVariant__data__)\
                                         \n        {\
                                         \n            .f1=(__math__MyVariant__data__f1__) { },\
                                         \n        },\
                                         \n    };\
                                         \n    math__MyVariant const v3 = (math__MyVariant)\
                                         \n    {\
                                         \n        .__kind__=__math__MyVariant__kind__f3__,\
                                         \n        .__data__=(__math__MyVariant__data__)\
                                         \n        {\
                                         \n            .f3=(__math__MyVariant__data__f3__)\
                                         \n            {\
                                         \n                .x=4,\
                                         \n                .y=7.5,\
                                         \n            },\
                                         \n        },\
                                         \n    };\
                                         \n}\n";

                let mut header_buffer = Vec::<u8>::new();
                let mut source_buffer = Vec::<u8>::new();
                let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

                writer.codegen_program(hir.as_ref()).unwrap();

                let header_res = String::from_utf8(header_buffer).unwrap();
                assert_str_eq!(HEADER_EXPECTED, header_res);

                let source_res = String::from_utf8(source_buffer).unwrap();
                assert_str_eq!(SOURCE_EXPECTED, source_res);
            }
        }

        #[test]
        fn denied_variants_test() {
            const SRC_TEXT: &str = "\nvariant MyVariant\
                                \n{\
                                \n    f1\
                                \n    f2(i32, i32)\
                                \n    f3 {\
                                \n        x: i32\
                                \n        y: f32\
                                \n    }\
                                \n}\
                                \nfunc main() {\
                                \n    var v1 = MyVariant::f1\
                                \n    var v3 = MyVariant::f3 {\
                                \n                 x: 4,\
                                \n                 y: 7.5\
                                \n             }\
                                \n}";

            let mut parser = Parser::from_text(SRC_TEXT);

            let ast = parser.parse_program().unwrap();

            let mut hir = {
                let mut lowering = AstLowering::new();
                lowering.low(ast.as_ref()).unwrap()
            };

            {
                const EXPECTED_ERR: &str = "Semantic error: Variants not supported in 0.1.0 (use \"--variants\" to enable variants)";

                let mut analyzer = Analyzer::new();
                let messages = analyzer.analyze_program(hir.as_mut()).err().unwrap();
                let errors = messages.errors_ref();

                assert!(!errors.is_empty());
                assert_str_eq!(errors[0].text, EXPECTED_ERR);
            }
        }
    */
}
