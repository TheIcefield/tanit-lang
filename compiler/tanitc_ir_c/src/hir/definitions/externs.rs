use tanitc_hir::hir::definitions::externs::ExternDef;

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_extern_def(&mut self, extern_def: &ExternDef) -> std::io::Result<()> {
        let mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        for func_def in extern_def.functions.iter() {
            self.generate_func_def(func_def, None)?;
        }

        self.mode = mode;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_hir::hir::{type_spec::Type, Hir};
    use tanitc_hir_test::create_func_def;

    use pretty_assertions::assert_str_eq;
    use tanitc_options::CompileOptions;

    #[test]
    fn extern_test() {
        // Given
        let mut func_def = create_func_def("c_func", vec![], Type::I32, vec![]);
        func_def.body = None;

        /*
         * extern "C" {
         *     func c_func() -> i32;
         * }
         */
        let program = Hir::from(ExternDef {
            abi_name: "C".to_string(),
            functions: vec![func_def],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::with_compile_options(
            &mut header_buffer,
            &mut source_buffer,
            CompileOptions {
                crate_name: "extern_test".into(),
                ..Default::default()
            },
        );

        writer.codegen_program(&program).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "signed int c_func();\n";
        const SOURCE_EXPECTED: &str = "#include \"extern_test.tt.h\"\n\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
