use tanitc_ast::ast::enums::EnumDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_enum_def(&mut self, enum_def: &EnumDef) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        let indentation = self.indentation();

        writeln!(self, "{indentation}typedef enum {{")?;

        for field in enum_def.fields.iter() {
            writeln!(
                self,
                "{indentation}    {} = {},",
                field.0,
                field.1.unwrap_or_default()
            )?;
        }

        writeln!(self, "{indentation}}} {};", enum_def.identifier)?;

        self.mode = old_mode;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        enums::{EnumDef, EnumUnits},
        Ast,
    };
    use tanitc_ident::Ident;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_enum(name: &str, user_units: Vec<(String, Option<usize>)>) -> EnumDef {
        let mut fields = EnumUnits::new();

        for (unit_name, unit_val) in user_units {
            fields.insert(Ident::from(unit_name), unit_val);
        }

        EnumDef {
            identifier: Ident::from(name.to_string()),
            fields,
            ..Default::default()
        }
    }

    #[test]
    fn empty_enum() {
        const ENUM_NAME: &str = "EmptyEnum";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n} EmptyEnum;\n";

        let node = Ast::from(get_enum(ENUM_NAME, Vec::new()));

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
    fn enum_with_1_unit() {
        const ENUM_NAME: &str = "MyEnum";
        const UNIT_1_NAME: &str = "A";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    A = 0,\
                                     \n} MyEnum;\n";

        let node = Ast::from(get_enum(ENUM_NAME, vec![(UNIT_1_NAME.to_string(), None)]));

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
    fn enum_with_3_units() {
        const ENUM_NAME: &str = "MyEnum";
        const UNIT_1_NAME: &str = "A";
        const UNIT_2_NAME: &str = "B";
        const UNIT_3_NAME: &str = "C";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    A = 4,\
                                     \n    B = 0,\
                                     \n    C = 0,\
                                     \n} MyEnum;\n";

        let node = Ast::from(get_enum(
            ENUM_NAME,
            vec![
                (UNIT_1_NAME.to_string(), Some(4)),
                (UNIT_2_NAME.to_string(), None),
                (UNIT_3_NAME.to_string(), None),
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
