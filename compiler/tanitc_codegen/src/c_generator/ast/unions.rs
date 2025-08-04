use tanitc_ast::ast::unions::UnionDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_union_def(&mut self, union_def: &UnionDef) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;
        let indentation = self.indentation();

        writeln!(self, "{indentation}typedef union {{")?;
        for (field_id, field_info) in union_def.fields.iter() {
            write!(self, "{indentation}    ")?;
            self.generate_type_spec(&field_info.ty)?;
            writeln!(self, " {field_id};")?;
        }
        writeln!(self, "}} {};", union_def.identifier)?;

        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        types::TypeSpec,
        unions::{UnionDef, UnionFieldInfo, UnionFields},
        Ast,
    };
    use tanitc_ident::Ident;
    use tanitc_ty::Type;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_union(name: &str, user_fields: Vec<(String, Type)>) -> UnionDef {
        let mut fields = UnionFields::new();

        for (field_name, field_ty) in user_fields {
            fields.insert(
                Ident::from(field_name),
                UnionFieldInfo {
                    ty: TypeSpec {
                        ty: field_ty,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        }

        UnionDef {
            identifier: Ident::from(name.to_string()),
            fields,
            ..Default::default()
        }
    }

    #[test]
    fn empty_union() {
        const UNION_NAME: &str = "EmptyUnion";
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \n} EmptyUnion;\n";

        let node = Ast::from(get_union(UNION_NAME, Vec::new()));

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
    fn union_with_1_field() {
        const UNION_NAME: &str = "MyUnion";
        const FIELD_1_NAME: &str = "a";
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \n    signed int a;\
                                     \n} MyUnion;\n";

        let node = Ast::from(get_union(
            UNION_NAME,
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
    fn union_with_3_fields() {
        const UNION_NAME: &str = "MyUnion";
        const FIELD_1_NAME: &str = "a";
        const FIELD_2_NAME: &str = "b";
        const FIELD_3_NAME: &str = "c";
        const FIELD_3_TYPE_NAME: &str = "C";
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \n    signed int a;\
                                     \n    float b;\
                                     \n    C c;\
                                     \n} MyUnion;\n";

        let node = Ast::from(get_union(
            UNION_NAME,
            vec![
                (FIELD_1_NAME.to_string(), Type::I32),
                (FIELD_2_NAME.to_string(), Type::F32),
                (
                    FIELD_3_NAME.to_string(),
                    Type::Custom(FIELD_3_TYPE_NAME.to_string()),
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
