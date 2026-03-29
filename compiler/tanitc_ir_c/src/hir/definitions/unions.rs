use tanitc_hir::hir::definitions::unions::UnionDef;

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_union_def(&mut self, union_def: &UnionDef) -> std::io::Result<()> {
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
        writeln!(self, "}} {};", union_def.name)?;

        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_hir::hir::type_spec::Type;
    use tanitc_hir_test::{create_custom_type, create_program, create_union_def};

    use pretty_assertions::assert_str_eq;

    #[test]
    fn empty_union() {
        // Given
        let union_def = create_union_def("EmptyUnion", vec![]);
        let node = create_program(vec![union_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        node.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \n} EmptyUnion;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn union_with_1_field() {
        // Given
        let union_def = create_union_def("MyUnion", vec![("a", Type::I32)]);
        let node = create_program(vec![union_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        node.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \n    signed int a;\
                                     \n} MyUnion;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }

    #[test]
    fn union_with_3_fields() {
        // Given
        let union_def = create_union_def(
            "MyUnion",
            vec![
                ("a", Type::I32),
                ("b", Type::F32),
                ("c", create_custom_type(&["C"])),
            ],
        );
        let node = create_program(vec![union_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        node.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \n    signed int a;\
                                     \n    float b;\
                                     \n    C c;\
                                     \n} MyUnion;\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
