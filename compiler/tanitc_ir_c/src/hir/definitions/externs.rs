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

    use tanitc_hir::hir::{definitions::functions::FunctionDef, types::Type, Hir};
    use tanitc_ident::Name;

    use pretty_assertions::assert_str_eq;

    #[test]
    fn extern_test() {
        const HEADER_EXPECTED: &str = "signed int c_func();\n";

        let extern_node = Hir::from(ExternDef {
            abi_name: "C".to_string(),
            functions: vec![FunctionDef {
                name: Name::from("c_func".to_string()),
                parameters: vec![],
                return_type: Type::I32,
                ..Default::default()
            }],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        extern_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
