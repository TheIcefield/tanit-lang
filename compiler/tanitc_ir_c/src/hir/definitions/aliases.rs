use tanitc_hir::hir::definitions::aliases::AliasDef;

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_alias_def(&mut self, alias_def: &AliasDef) -> std::io::Result<()> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        write!(self, "typedef ")?;
        self.generate_type_spec(&alias_def.value)?;

        write!(self, " ")?;

        self.generate_name_spec(&alias_def.name)?;
        writeln!(self, ";")?;

        self.mode = old_mode;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_hir::hir::{blocks::Block, type_spec::Type, Hir};
    use tanitc_hir_test::{create_alias_def, create_custom_type, create_struct_def};

    use pretty_assertions::assert_str_eq;

    #[test]
    fn codegen_simple_alias_test() {
        // Given
        const ALIAS_NAME: &str = "MyAlias";

        /*
         * alias MyAlias = f32
         */
        let node = Hir::from(Block {
            is_global: true,
            statements: vec![create_alias_def(ALIAS_NAME, Type::F32).into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        node.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef float MyAlias;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn codegen_alias_test() {
        // Given
        const STRUCT_NAME: &str = "EmptyStruct";
        const ALIAS_1_NAME: &str = "FirstAlias";
        const ALIAS_2_NAME: &str = "SecondAlias";

        /*
         * alias FirstAlias = EmptyStruct
         * alias SecondAlias = FirstAlias
         */
        let node = Hir::from(Block {
            is_global: true,
            statements: vec![
                create_struct_def(STRUCT_NAME, vec![]).into(),
                create_alias_def(ALIAS_1_NAME, create_custom_type(&[STRUCT_NAME])).into(),
                create_alias_def(ALIAS_2_NAME, create_custom_type(&[ALIAS_1_NAME])).into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        node.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} EmptyStruct;\
                                     \ntypedef EmptyStruct FirstAlias;\
                                     \ntypedef FirstAlias SecondAlias;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
