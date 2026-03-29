use std::collections::BTreeMap;

use crate::symbol_table::entry::{
    Entry, StructFieldData, StructFieldsData, VariantData, VariantDefData, VariantKind,
    VariantStruct, VariantTuple,
};
use tanitc_hir::hir::{
    definitions::{
        structs::StructFieldsInfo,
        variants::{VariantDef, VariantField},
    },
    type_spec::Type,
};
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_name::NameSpec;

use crate::{AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_variant_def(
        &mut self,
        variant_def: &mut VariantDef,
    ) -> AnalyzeResult<()> {
        self.check_variants_are_allowed(variant_def.location)?;

        let variant_id = variant_def
            .name
            .get_id()
            .ok_or(Message::empty_name_spec(variant_def.location))?;

        if self.has_symbol(variant_id) {
            return Err(Message::multiple_ids(variant_def.location, variant_id));
        }

        variant_def.name.path.splice(0..0, self.table.get_path());

        let variants = self.get_variants_from_definition(variant_def)?;
        let variant_def_data = VariantDefData {
            name: variant_def.name.clone(),
            variants,
        };
        let entry = Entry {
            id: variant_id,
            is_static: true,
            kind: variant_def_data.into(),
        };

        self.add_symbol(entry);

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

    fn get_struct_variant_kind(
        &self,
        variant_name: &NameSpec,
        variant_struct_fields: &StructFieldsInfo,
    ) -> VariantKind {
        let mut variant_fields = StructFieldsData::new();
        for (field_name, field_ty) in variant_struct_fields.iter() {
            variant_fields.insert(
                *field_name,
                StructFieldData {
                    name: NameSpec::default(),
                    ty: field_ty.ty.get_type(),
                },
            );
        }

        VariantKind::Struct(VariantStruct {
            name: variant_name.clone(),
            fields: variant_fields,
        })
    }

    fn get_tuple_variant_kind(&self, variant_tuple_components: &[Type]) -> VariantKind {
        let mut variant_fields = BTreeMap::<usize, StructFieldData>::new();
        for (field_num, field_ty) in variant_tuple_components.iter().enumerate() {
            variant_fields.insert(
                field_num,
                StructFieldData {
                    name: NameSpec::default(),
                    ty: field_ty.clone(),
                },
            );
        }

        VariantKind::Tuple(VariantTuple {
            fields: variant_fields,
        })
    }

    fn get_variants_from_definition(
        &self,
        variant_def: &VariantDef,
    ) -> Result<BTreeMap<Ident, Entry>, Message> {
        let mut res: BTreeMap<Ident, Entry> = BTreeMap::new();

        for (variant_kind_num, (variant_unit_id, variant)) in variant_def.fields.iter().enumerate()
        {
            let variant_name = variant_def.name.clone();

            let variant_data = VariantData {
                variant_kind: match variant {
                    VariantField::Enum => VariantKind::Enum,
                    VariantField::Tuple(fields) => self.get_tuple_variant_kind(fields),
                    VariantField::Struct(fields) => {
                        self.get_struct_variant_kind(&variant_name, fields)
                    }
                },
                variant_unit_id: *variant_unit_id,
                variant_kind_num,
                variant_name,
            };

            res.insert(
                *variant_unit_id,
                Entry {
                    id: *variant_unit_id,
                    is_static: true,
                    kind: variant_data.into(),
                },
            );
        }

        Ok(res)
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
