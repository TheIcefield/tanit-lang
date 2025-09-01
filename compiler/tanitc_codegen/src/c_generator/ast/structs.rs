use tanitc_ast::ast::structs::StructDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_struct_def(&mut self, struct_def: &StructDef) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;
        let indentation = self.indentation();

        writeln!(self, "{indentation}typedef struct {{")?;
        for (field_id, field_info) in struct_def.fields.iter() {
            write!(self, "{indentation}    ")?;
            self.generate_type_spec(&field_info.ty)?;
            writeln!(self, " {field_id};")?;
        }
        writeln!(self, "{indentation}}} {};", struct_def.name)?;

        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        structs::{StructDef, StructFieldInfo, StructFields},
        types::TypeSpec,
        Ast,
    };
    use tanitc_ident::{Ident, Name};
    use tanitc_ty::Type;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_struct(name: &str, user_fields: Vec<(String, Type)>) -> StructDef {
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

        StructDef {
            name: Name::from(name.to_string()),
            fields,
            ..Default::default()
        }
    }

    #[test]
    fn empty_struct() {
        const STRUCT_NAME: &str = "EmptyStruct";
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} EmptyStruct;\n";

        let node = Ast::from(get_struct(STRUCT_NAME, Vec::new()));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn struct_with_1_field() {
        const STRUCT_NAME: &str = "MyStruct";
        const FIELD_1_NAME: &str = "a";
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n    signed int a;\
                                     \n} MyStruct;\n";

        let node = Ast::from(get_struct(
            STRUCT_NAME,
            vec![(FIELD_1_NAME.to_string(), Type::I32)],
        ));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn struct_with_3_fields() {
        const STRUCT_NAME: &str = "MyStruct";
        const FIELD_1_NAME: &str = "a";
        const FIELD_2_NAME: &str = "b";
        const FIELD_3_NAME: &str = "c";
        const FIELD_3_TYPE_NAME: &str = "C";
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n    signed int a;\
                                     \n    float b;\
                                     \n    C c;\
                                     \n} MyStruct;\n";

        let node = Ast::from(get_struct(
            STRUCT_NAME,
            vec![
                (FIELD_1_NAME.to_string(), Type::I32),
                (FIELD_2_NAME.to_string(), Type::F32),
                (
                    FIELD_3_NAME.to_string(),
                    Type::Custom(Name::from(FIELD_3_TYPE_NAME.to_string())),
                ),
            ],
        ));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
