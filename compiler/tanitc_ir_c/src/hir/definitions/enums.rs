use tanitc_hir::hir::definitions::enums::EnumDef;

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_enum_def(&mut self, enum_def: &EnumDef) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        let indentation = self.indentation();

        writeln!(self, "{indentation}typedef enum {{")?;

        for field in enum_def.units.iter() {
            writeln!(
                self,
                "{indentation}    {} = {},",
                field.0,
                field.1.unwrap_or_default()
            )?;
        }

        writeln!(self, "{indentation}}} {};", enum_def.name)?;

        self.mode = old_mode;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_str_eq;
    use tanitc_hir_test::{create_enum_def, create_program};
    use tanitc_options::CompileOptions;

    #[test]
    fn empty_enum() {
        // Given
        let enum_def = create_enum_def("EmptyEnum", vec![]);

        let program = create_program(vec![enum_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "EnumsTest".into(),
                ..Default::default()
            },
        );

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const SOURCE_EXPECTED: &str = "#include \"EnumsTest.tt.h\"\n\n";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n} EmptyEnum;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn enum_with_1_unit() {
        // Given
        let enum_def = create_enum_def("MyEnum", vec![("A", None)]);
        let program = create_program(vec![enum_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "EnumsTest".into(),
                ..Default::default()
            },
        );

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const SOURCE_EXPECTED: &str = "#include \"EnumsTest.tt.h\"\n\n";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    A = 0,\
                                     \n} MyEnum;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn enum_with_3_units() {
        // Given
        let enum_def = create_enum_def("MyEnum", vec![("A", Some(4)), ("B", None), ("C", None)]);
        let program = create_program(vec![enum_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "EnumsTest".into(),
                ..Default::default()
            },
        );

        // When
        writer.codegen_program(&program).unwrap();

        // Then
        const SOURCE_EXPECTED: &str = "#include \"EnumsTest.tt.h\"\n\n";
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    A = 4,\
                                     \n    B = 0,\
                                     \n    C = 0,\
                                     \n} MyEnum;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
