use tanitc_ast::ast::externs::ExternDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_extern_def(&mut self, extern_def: &ExternDef) -> Result<(), std::io::Error> {
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
    use tanitc_ast::ast::{externs::ExternDef, functions::FunctionDef, types::TypeSpec, Ast};
    use tanitc_ident::Ident;
    use tanitc_ty::Type;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    #[test]
    fn extern_test() {
        const HEADER_EXPECTED: &str = "signed int c_func();\n";

        let extern_node = Ast::from(ExternDef {
            abi_name: "C".to_string(),
            functions: vec![FunctionDef {
                identifier: Ident::from("c_func".to_string()),
                parameters: vec![],
                return_type: TypeSpec {
                    ty: Type::I32,
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        extern_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
