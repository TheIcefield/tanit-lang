use tanitc_ast::ast::{
    structs::StructFields,
    types::TypeSpec,
    variants::{
        get_variant_data_kind_id, get_variant_data_type_id, VariantDef, VariantField, VariantFields,
    },
};
use tanitc_ident::Ident;

use crate::c_generator::{CodeGenMode, CodeGenStream};

use std::{collections::BTreeMap, io::Write};

impl CodeGenStream<'_> {
    pub fn generate_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        self.generate_variant_kind(variant_def.identifier, &variant_def.fields)?;
        self.generate_variant_data(variant_def.identifier, &variant_def.fields)?;

        writeln!(self, "typedef struct {{")?;

        self.generate_variant_kind_field(variant_def.identifier)?;
        self.generate_variant_data_field(variant_def.identifier)?;

        writeln!(self, "}} {};\n", variant_def.identifier)?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_variant_kind(
        &mut self,
        variant_id: Ident,
        fields: &VariantFields,
    ) -> Result<(), std::io::Error> {
        let enum_id = get_variant_data_kind_id(variant_id);

        // Enum definition
        writeln!(self, "typedef enum {{")?;
        for (field_id, _) in fields.iter() {
            writeln!(self, "    {enum_id}{field_id}__,")?;
        }
        writeln!(self, "}} {enum_id};\n")?;

        Ok(())
    }

    fn generate_variant_kind_field(&mut self, variant_id: Ident) -> Result<(), std::io::Error> {
        let enum_id = get_variant_data_kind_id(variant_id);
        let field_id = Ident::from("__kind__".to_string());

        writeln!(self, "    {enum_id} {field_id};")?;

        Ok(())
    }

    fn generate_variant_common_field(
        &mut self,
        union_id: Ident,
        field_id: Ident,
    ) -> Result<(), std::io::Error> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{ }} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_struct_field(
        &mut self,
        union_id: Ident,
        field_id: Ident,
        subfields: &StructFields,
    ) -> Result<(), std::io::Error> {
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
        union_id: Ident,
        field_id: Ident,
        components: &[TypeSpec],
    ) -> Result<(), std::io::Error> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{")?;

        for (field_num, field_type) in components.iter().enumerate() {
            let field_type = field_type.get_type().get_c_type();
            writeln!(self, "    {field_type} _{field_num};")?;
        }

        writeln!(self, "}} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_data_types(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        let union_id = get_variant_data_type_id(variant_id);

        for (field_id, field_data) in fields.iter() {
            match field_data {
                VariantField::Common => self.generate_variant_common_field(union_id, *field_id)?,
                VariantField::StructLike(subfields) => {
                    self.generate_variant_struct_field(union_id, *field_id, subfields)?
                }
                VariantField::TupleLike(components) => {
                    self.generate_variant_tuple_field(union_id, *field_id, components)?
                }
            }
            writeln!(self)?;
        }

        Ok(())
    }

    fn generate_variant_data_fields(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        let union_id = get_variant_data_type_id(variant_id);

        writeln!(self, "typedef union {union_id} {{")?;

        for (field_id, _) in fields.iter() {
            writeln!(self, "    {union_id}{field_id}__ {field_id};")?;
        }

        writeln!(self, "}} {union_id};\n")?;

        Ok(())
    }

    fn generate_variant_data(
        &mut self,
        variant_id: Ident,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        self.generate_variant_data_types(variant_id, fields)?;
        self.generate_variant_data_fields(variant_id, fields)?;

        Ok(())
    }

    fn generate_variant_data_field(&mut self, variant_id: Ident) -> Result<(), std::io::Error> {
        let union_id = get_variant_data_type_id(variant_id);
        let field_id = Ident::from("__data__".to_string());

        writeln!(self, "    {union_id} {field_id};")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        structs::{StructFieldInfo, StructFields},
        types::TypeSpec,
        variants::{VariantDef, VariantField, VariantFields},
        Ast,
    };
    use tanitc_ident::Ident;
    use tanitc_ty::Type;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_cmn_field(name: &str) -> (Ident, VariantField) {
        (Ident::from(name.to_string()), VariantField::Common)
    }

    fn get_struct_field(name: &str, user_fields: Vec<(String, Type)>) -> (Ident, VariantField) {
        let mut fields = StructFields::new();

        for (field_name, field_ty) in user_fields {
            fields.insert(
                Ident::from(field_name),
                StructFieldInfo {
                    ty: TypeSpec {
                        ty: field_ty,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        }

        (
            Ident::from(name.to_string()),
            VariantField::StructLike(fields),
        )
    }

    fn get_tuple_field(name: &str, user_fields: Vec<Type>) -> (Ident, VariantField) {
        let mut fields = Vec::<TypeSpec>::new();

        for field_ty in user_fields {
            fields.push(TypeSpec {
                ty: field_ty,
                ..Default::default()
            });
        }

        (
            Ident::from(name.to_string()),
            VariantField::TupleLike(fields),
        )
    }

    fn get_variant_def(name: &str, user_fields: Vec<(Ident, VariantField)>) -> VariantDef {
        let mut fields = VariantFields::new();

        for (field_name, field) in user_fields {
            fields.insert(field_name, field);
        }

        VariantDef {
            identifier: Ident::from(name.to_string()),
            fields,
            ..Default::default()
        }
    }

    #[test]
    fn empty_variant_test() {
        const VARIANT_NAME: &str = "MyVariant";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n} __MyVariant__kind__;\
                                     \n\
                                     \ntypedef union __MyVariant__data__ {\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\n\n";

        let program = Ast::from(get_variant_def(VARIANT_NAME, vec![]));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn enum_variant_test() {
        const VARIANT_NAME: &str = "MyVariant";
        const UNIT_1_NAME: &str = "A";
        const UNIT_2_NAME: &str = "B";
        const UNIT_3_NAME: &str = "C";
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
                                     \ntypedef union __MyVariant__data__ {\
                                     \n    __MyVariant__data__A__ A;\
                                     \n    __MyVariant__data__B__ B;\
                                     \n    __MyVariant__data__C__ C;\
                                     \n} __MyVariant__data__;\
                                     \n\
                                     \ntypedef struct {\
                                     \n    __MyVariant__kind__ __kind__;\
                                     \n    __MyVariant__data__ __data__;\
                                     \n} MyVariant;\n\n";

        let program = Ast::from(get_variant_def(
            VARIANT_NAME,
            vec![
                get_cmn_field(UNIT_1_NAME),
                get_cmn_field(UNIT_2_NAME),
                get_cmn_field(UNIT_3_NAME),
            ],
        ));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn full_variant_test() {
        const VARIANT_NAME: &str = "MyVariant";
        const UNIT_1_NAME: &str = "A";
        const UNIT_2_NAME: &str = "B";
        const UNIT_3_NAME: &str = "C";
        const UNIT_3_1_NAME: &str = "c";
        const UNIT_4_NAME: &str = "D";
        const UNIT_4_1_NAME: &str = "d1";
        const UNIT_4_2_NAME: &str = "d2";
        const UNIT_5_NAME: &str = "E";
        const UNIT_6_NAME: &str = "F";
        const UNIT_7_NAME: &str = "G";

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
                                     \ntypedef union __MyVariant__data__ {\
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

        let program = Ast::from(get_variant_def(
            VARIANT_NAME,
            vec![
                get_cmn_field(UNIT_1_NAME),
                get_struct_field(UNIT_2_NAME, vec![]),
                get_struct_field(UNIT_3_NAME, vec![(UNIT_3_1_NAME.to_string(), Type::F32)]),
                get_struct_field(
                    UNIT_4_NAME,
                    vec![
                        (UNIT_4_1_NAME.to_string(), Type::Bool),
                        (UNIT_4_2_NAME.to_string(), Type::I32),
                    ],
                ),
                get_tuple_field(UNIT_5_NAME, vec![]),
                get_tuple_field(UNIT_6_NAME, vec![Type::F64]),
                get_tuple_field(UNIT_7_NAME, vec![Type::I8, Type::I16, Type::I32]),
            ],
        ));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
