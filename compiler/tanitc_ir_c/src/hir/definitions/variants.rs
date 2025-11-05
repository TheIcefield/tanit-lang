use tanitc_hir::hir::{
    definitions::{
        structs::StructFieldsInfo,
        variants::{
            get_variant_data_kind_id, get_variant_data_type_id, VariantDef, VariantField,
            VariantFields,
        },
    },
    types::Type,
};
use tanitc_ident::{Ident, Name};

use crate::{CodeGenMode, CodeGenStream};

use std::{collections::BTreeMap, io::Write};

impl CodeGenStream<'_> {
    pub fn generate_variant_def(&mut self, variant_def: &VariantDef) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        self.generate_variant_kind(variant_def.name, &variant_def.fields)?;
        self.generate_variant_data(variant_def.name, &variant_def.fields)?;

        writeln!(self, "typedef struct {{")?;

        self.generate_variant_kind_field(variant_def.name)?;
        self.generate_variant_data_field(variant_def.name)?;

        writeln!(self, "}} {};\n", variant_def.name)?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_variant_kind(
        &mut self,
        variant_id: Name,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        let enum_id = get_variant_data_kind_id(variant_id);

        // Enum definition
        writeln!(self, "typedef enum {{")?;
        for (field_id, _) in fields.iter() {
            writeln!(self, "    {enum_id}{field_id}__,")?;
        }
        writeln!(self, "}} {enum_id};\n")?;

        Ok(())
    }

    fn generate_variant_kind_field(&mut self, variant_id: Name) -> Result<(), std::io::Error> {
        let enum_id = get_variant_data_kind_id(variant_id);
        let field_id = Ident::from("__kind__".to_string());

        writeln!(self, "    {enum_id} {field_id};")?;

        Ok(())
    }

    fn generate_variant_enum_field(
        &mut self,
        union_id: Name,
        field_id: Ident,
    ) -> std::io::Result<()> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{ }} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_struct_field(
        &mut self,
        union_id: Name,
        field_id: Ident,
        subfields: &StructFieldsInfo,
    ) -> std::io::Result<()> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{")?;

        for (subfield_id, subfield_type) in subfields.iter() {
            let subfield_type = subfield_type.ty.get_type().get_c_type();
            writeln!(self, "    {subfield_type} {subfield_id};")?;
        }

        writeln!(self, "}} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_tuple_field(
        &mut self,
        union_id: Name,
        field_id: Ident,
        components: &[Type],
    ) -> std::io::Result<()> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{")?;

        for (field_num, field_type) in components.iter().enumerate() {
            let field_type = field_type.get_c_type();
            writeln!(self, "    {field_type} _{field_num};")?;
        }

        writeln!(self, "}} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_data_types(
        &mut self,
        variant_id: Name,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        let union_id = get_variant_data_type_id(variant_id);

        for (field_id, field_data) in fields.iter() {
            match field_data {
                VariantField::Enum => self.generate_variant_enum_field(union_id, *field_id)?,
                VariantField::Struct(subfields) => {
                    self.generate_variant_struct_field(union_id, *field_id, subfields)?
                }
                VariantField::Tuple(components) => {
                    self.generate_variant_tuple_field(union_id, *field_id, components)?
                }
            }
            writeln!(self)?;
        }

        Ok(())
    }

    fn generate_variant_data_fields(
        &mut self,
        variant_id: Name,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        let union_id = get_variant_data_type_id(variant_id);

        writeln!(self, "typedef union {{")?;

        for (field_id, _) in fields.iter() {
            writeln!(self, "    {union_id}{field_id}__ {field_id};")?;
        }

        writeln!(self, "}} {union_id};\n")?;

        Ok(())
    }

    fn generate_variant_data(
        &mut self,
        variant_id: Name,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> std::io::Result<()> {
        self.generate_variant_data_types(variant_id, fields)?;
        self.generate_variant_data_fields(variant_id, fields)?;

        Ok(())
    }

    fn generate_variant_data_field(&mut self, variant_id: Name) -> std::io::Result<()> {
        let union_id = get_variant_data_type_id(variant_id);
        let field_id = Ident::from("__data__".to_string());

        writeln!(self, "    {union_id} {field_id};")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_hir::hir::Hir;
    use tanitc_hir_test::{
        create_enum_variantfield, create_struct_variantfield, create_tuple_variantfield,
        create_variant_def,
    };

    use pretty_assertions::assert_str_eq;
    use tanitc_options::CompileOptions;

    use super::*;

    #[test]
    fn empty_variant_test() {
        // Given
        let program = Hir::from(create_variant_def("MyVariant", vec![]));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "VariantsTest".into(),
                ..Default::default()
            },
        );

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const SOURCE_EXPECTED: &str = "#include \"VariantsTest.tt.h\"\n\n";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \ntypedef union {\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\n\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn enum_variant_test() {
        // Given
        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "VariantsTest".into(),
                ..Default::default()
            },
        );

        /* variant MyVariant {
         *     A
         *     B
         *     C
         * }
         */
        let program = Hir::from(create_variant_def(
            "MyVariant",
            vec![
                create_enum_variantfield("A"),
                create_enum_variantfield("B"),
                create_enum_variantfield("C"),
            ],
        ));

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const SOURCE_EXPECTED: &str = "#include \"VariantsTest.tt.h\"\n\n";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    __MyVariant__kind__A__,\
                                     \n    __MyVariant__kind__B__,\
                                     \n    __MyVariant__kind__C__,\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \ntypedef struct { } __MyVariant__data__A__;\
                                     \n\
                                     \ntypedef struct { } __MyVariant__data__B__;\
                                     \n\
                                     \ntypedef struct { } __MyVariant__data__C__;\
                                     \n\
                                     \ntypedef union {\
                                     \n    __MyVariant__data__A__ A;\
                                     \n    __MyVariant__data__B__ B;\
                                     \n    __MyVariant__data__C__ C;\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\n\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn full_variant_test() {
        // Given
        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "VariantsTest".into(),
                ..Default::default()
            },
        );

        /*
         * variant MyVariant {
         *     A
         *     B { }
         *     C { c: f32 }
         *     D { d1: bool, d2: i32 }
         *     E()
         *     F(f64)
         *     G(i8, i16, i32)
         * }
         */
        let program = Hir::from(create_variant_def(
            "MyVariant",
            vec![
                create_enum_variantfield("A"),
                create_struct_variantfield("B", vec![]),
                create_struct_variantfield("C", vec![("c", Type::F32)]),
                create_struct_variantfield("D", vec![("d1", Type::Bool), ("d2", Type::I32)]),
                create_tuple_variantfield("E", vec![]),
                create_tuple_variantfield("F", vec![Type::F64]),
                create_tuple_variantfield("G", vec![Type::I8, Type::I16, Type::I32]),
            ],
        ));

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const SOURCE_EXPECTED: &str = "#include \"VariantsTest.tt.h\"\n\n";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    __MyVariant__kind__A__,\
                                     \n    __MyVariant__kind__B__,\
                                     \n    __MyVariant__kind__C__,\
                                     \n    __MyVariant__kind__D__,\
                                     \n    __MyVariant__kind__E__,\
                                     \n    __MyVariant__kind__F__,\
                                     \n    __MyVariant__kind__G__,\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \ntypedef struct { } __MyVariant__data__A__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n} __MyVariant__data__B__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    float c;\
                                     \n} __MyVariant__data__C__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    unsigned char d1;\
                                     \n    signed int d2;\
                                     \n} __MyVariant__data__D__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n} __MyVariant__data__E__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    double _0;\
                                     \n} __MyVariant__data__F__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    unsigned int _0;\
                                     \n    signed short _1;\
                                     \n    signed int _2;\
                                     \n} __MyVariant__data__G__;\
                                     \n\
                                     \ntypedef union {\
                                     \n    __MyVariant__data__A__ A;\
                                     \n    __MyVariant__data__B__ B;\
                                     \n    __MyVariant__data__C__ C;\
                                     \n    __MyVariant__data__D__ D;\
                                     \n    __MyVariant__data__E__ E;\
                                     \n    __MyVariant__data__F__ F;\
                                     \n    __MyVariant__data__G__ G;\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\n\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
