use tanitc_ast::ast::{
    blocks::Block,
    values::{Value, ValueKind},
    Ast,
};

use crate::c_generator::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_block(&mut self, block: &Block) -> Result<(), std::io::Error> {
        use std::io::Write;

        let indentation = self.indentation();

        if !block.is_global {
            writeln!(self, "{indentation}{{")?;
            self.indent += 1;
        }

        for stmt in block.statements.iter() {
            if !matches!(stmt, Ast::Block(_)) {
                write!(self, "{indentation}    ")?;
            }

            self.generate(stmt)?;

            match stmt {
                Ast::Expression(_)
                | Ast::ControlFlow(_)
                | Ast::VariableDef(_)
                | Ast::Value(Value {
                    kind: ValueKind::Call { .. },
                    ..
                }) => write!(self, ";")?,
                _ => {}
            }

            writeln!(self)?;
        }

        if !block.is_global {
            writeln!(self, "{indentation}}}")?;
            self.indent -= 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{blocks::Block, functions::FunctionDef, Ast};

    use pretty_assertions::assert_str_eq;
    use tanitc_ident::Ident;

    use crate::c_generator::CodeGenStream;

    fn get_block(is_global: bool, statements: Vec<Ast>) -> Block {
        Block {
            statements,
            is_global,
            ..Default::default()
        }
    }

    #[test]
    fn codegen_block_test() {
        const HEADER_EXPECTED: &str = "void hello();\n";
        const SOURCE_EXPECTED: &str = "void hello()\
                                     \n{\
                                     \n    {\
                                     \n    }\
                                     \n\
                                     \n    {\
                                     \n    }\
                                     \n\
                                     \n}\n";

        let node = Ast::from(FunctionDef {
            identifier: Ident::from("hello".to_string()),
            body: Some(Box::new(get_block(
                false,
                vec![
                    get_block(false, vec![]).into(),
                    get_block(false, vec![]).into(),
                ],
            ))),
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
