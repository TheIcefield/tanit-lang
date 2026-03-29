use tanitc_hir::hir::{
    definitions::{
        structs::StructFieldsInfo,
        variants::{VariantDef, VariantField, VariantFields},
    },
    type_spec::Type,
};
use tanitc_ident::Ident;
use tanitc_name::NameSpec;

use crate::{CodeGenMode, CodeGenStream};

use std::io::Write;

impl CodeGenStream<'_> {
    /* Generates:
     * typedef enum {
     *     __<variant_id>__kind__<fields[0].0>;
     *     __<variant_id>__kind__<fields[1].0>;
     *     ...
     *     __<variant_id>__kind__<fields[N].0>;
     * } __<variant_id>__kind__;
     *
     * typedef union {
     *
     * } __<variant_name>__data__;
     */
    pub fn generate_variant_def(&mut self, variant_def: &VariantDef) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        writeln!(
            self,
            "// Generated structs/enums for variant {}",
            variant_def.name
        )?;

        self.generate_variant_kind(&variant_def.name, &variant_def.fields)?;
        self.generate_variant_data(&variant_def.name, &variant_def.fields)?;

        writeln!(self, "typedef struct {{")?;

        self.generate_variant_kind_field(&variant_def.name)?;
        self.generate_variant_data_field(&variant_def.name)?;

        write!(self, "}} ")?;
        self.generate_name_spec(&variant_def.name)?;
        writeln!(self, ";\n")?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_variant_kind(
        &mut self,
        variant_id: &NameSpec,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        let enum_name = VariantDef::get_variant_data_kind_name(variant_id);

        writeln!(self, "typedef enum {{")?;
        for (field_id, _) in fields.iter() {
            write!(self, "    __")?;
            self.generate_name_spec(&enum_name)?;
            writeln!(self, "__{field_id}__,")?;
        }

        write!(self, "}} __")?;
        self.generate_name_spec(&enum_name)?;
        writeln!(self, "__;\n")?;

        Ok(())
    }

    fn generate_variant_data(
        &mut self,
        variant_name: &NameSpec,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        self.generate_variant_data_types(variant_name, fields)?;
        self.generate_variant_data_fields(variant_name, fields)?;

        Ok(())
    }

    fn generate_variant_data_fields(
        &mut self,
        variant_name: &NameSpec,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        let union_name = VariantDef::get_variant_data_type_name(variant_name);

        writeln!(self, "// union type that stores any data of variant fields")?;
        writeln!(self, "typedef union {{")?;

        for (field_id, _) in fields.iter() {
            write!(self, "    __")?;
            self.generate_name_spec(&union_name)?;
            writeln!(self, "__{field_id}__ {field_id};")?;
        }

        write!(self, "}} __")?;
        self.generate_name_spec(&union_name)?;
        writeln!(self, "__;\n")?;

        Ok(())
    }

    fn generate_variant_kind_field(
        &mut self,
        variant_name: &NameSpec,
    ) -> Result<(), std::io::Error> {
        let enum_name = VariantDef::get_variant_data_kind_name(variant_name);

        writeln!(self, "    // Used to identifie variant kind in run-time")?;

        write!(self, "    __")?;
        self.generate_name_spec(&enum_name)?;
        writeln!(self, "__ __kind__;")?;

        Ok(())
    }

    // Generates empty struct for variant unit of kind: enum
    fn generate_variant_enum_field(
        &mut self,
        union_name: &NameSpec,
        field_id: Ident,
    ) -> std::io::Result<()> {
        writeln!(self, "// Empty struct for enum-variant: {field_id}")?;
        write!(self, "typedef struct {{ }} __")?;
        self.generate_name_spec(union_name)?;
        writeln!(self, "__{field_id}__;")?;

        Ok(())
    }

    // Generates struct for variant unit of kind: struct
    fn generate_variant_struct_field(
        &mut self,
        union_name: &NameSpec,
        field_id: Ident,
        subfields: &StructFieldsInfo,
    ) -> std::io::Result<()> {
        writeln!(self, "// Struct for struct-variant: {field_id}")?;
        writeln!(self, "typedef struct {{")?;

        for (subfield_id, subfield_type) in subfields.iter() {
            let subfield_type = subfield_type.ty.get_type().get_c_type();
            writeln!(self, "    {subfield_type} {subfield_id};")?;
        }

        write!(self, "}} __")?;
        self.generate_name_spec(union_name)?;
        writeln!(self, "__{field_id}__;")?;

        Ok(())
    }

    // Generates struct for variant unit of kind: tuple
    fn generate_variant_tuple_field(
        &mut self,
        union_name: &NameSpec,
        field_id: Ident,
        components: &[Type],
    ) -> std::io::Result<()> {
        writeln!(self, "// Struct for tuple-variant: {field_id}")?;
        writeln!(self, "typedef struct {{")?;

        for (field_num, field_type) in components.iter().enumerate() {
            let field_type = field_type.get_c_type();
            writeln!(self, "    {field_type} _{field_num};")?;
        }

        write!(self, "}} __")?;
        self.generate_name_spec(union_name)?;
        writeln!(self, "__{field_id}__;")?;

        Ok(())
    }

    fn generate_variant_data_types(
        &mut self,
        variant_name: &NameSpec,
        fields: &VariantFields,
    ) -> std::io::Result<()> {
        let union_name = VariantDef::get_variant_data_type_name(variant_name);

        for (field_id, field_data) in fields.iter() {
            writeln!(self, "// FIELD {field_id}")?;
            match field_data {
                VariantField::Enum => self.generate_variant_enum_field(&union_name, *field_id)?,
                VariantField::Struct(subfields) => {
                    self.generate_variant_struct_field(&union_name, *field_id, subfields)?
                }
                VariantField::Tuple(components) => {
                    self.generate_variant_tuple_field(&union_name, *field_id, components)?
                }
            }
            writeln!(self)?;
        }

        Ok(())
    }

    fn generate_variant_data_field(&mut self, variant_name: &NameSpec) -> std::io::Result<()> {
        let union_name = VariantDef::get_variant_data_type_name(variant_name);

        writeln!(self, "    // Used to store variant data in run-time")?;

        write!(self, "    __")?;
        self.generate_name_spec(&union_name)?;
        writeln!(self, "__ __data__;")?;

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
        const HEADER_EXPECTED: &str = "// Generated structs/enums for variant MyVariant\
                                     \ntypedef enum {\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \n// union type that stores any data of variant fields\
                                     \ntypedef union {\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    // Used to identifie variant kind in run-time\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    // Used to store variant data in run-time\
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
        const HEADER_EXPECTED: &str = "// Generated structs/enums for variant MyVariant\
                                     \ntypedef enum {\
                                     \n    __MyVariant__kind__A__,\
                                     \n    __MyVariant__kind__B__,\
                                     \n    __MyVariant__kind__C__,\
                                     \n    __MyVariant__kind__D__,\
                                     \n    __MyVariant__kind__E__,\
                                     \n    __MyVariant__kind__F__,\
                                     \n    __MyVariant__kind__G__,\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \n// FIELD A\
                                     \n// Empty struct for enum-variant: A\
                                     \ntypedef struct { } __MyVariant__data__A__;\
                                     \n\
                                     \n// FIELD B\
                                     \n// Struct for struct-variant: B\
                                     \ntypedef struct {\
                                     \n} __MyVariant__data__B__;\
                                     \n\
                                     \n// FIELD C\
                                     \n// Struct for struct-variant: C\
                                     \ntypedef struct {\
                                     \n    float c;\
                                     \n} __MyVariant__data__C__;\
                                     \n\
                                     \n// FIELD D\
                                     \n// Struct for struct-variant: D\
                                     \ntypedef struct {\
                                     \n    unsigned char d1;\
                                     \n    signed int d2;\
                                     \n} __MyVariant__data__D__;\
                                     \n\
                                     \n// FIELD E\
                                     \n// Struct for tuple-variant: E\
                                     \ntypedef struct {\
                                     \n} __MyVariant__data__E__;\
                                     \n\
                                     \n// FIELD F\
                                     \n// Struct for tuple-variant: F\
                                     \ntypedef struct {\
                                     \n    double _0;\
                                     \n} __MyVariant__data__F__;\
                                     \n\
                                     \n// FIELD G\
                                     \n// Struct for tuple-variant: G\
                                     \ntypedef struct {\
                                     \n    unsigned int _0;\
                                     \n    signed short _1;\
                                     \n    signed int _2;\
                                     \n} __MyVariant__data__G__;\
                                     \n\
                                     \n// union type that stores any data of variant fields\
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
                                     \n    // Used to identifie variant kind in run-time\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    // Used to store variant data in run-time\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\n\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
