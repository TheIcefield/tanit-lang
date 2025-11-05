use tanitc_hir::hir::{blocks::Block, definitions::Definition, Hir};

use crate::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_block(&mut self, block: &Block) -> std::io::Result<()> {
        use std::io::Write;

        let indentation = self.indentation();

        if !block.is_global {
            writeln!(self, "{indentation}{{")?;
            self.indent += 1;
        }

        for stmt in block.statements.iter() {
            if !matches!(stmt, Hir::Block(_)) {
                write!(self, "{indentation}    ")?;
            }

            self.generate(stmt)?;

            match stmt {
                Hir::Expression(_)
                | Hir::ControlFlow(_)
                | Hir::Definition(Definition::Variable(_)) => write!(self, ";")?,
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
    use super::*;

    use tanitc_hir::hir::{blocks::Block, definitions::functions::FunctionDef, Hir};
    use tanitc_ident::Name;

    use pretty_assertions::assert_str_eq;

    fn get_block(is_global: bool, statements: Vec<Hir>) -> Block {
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

        let node = Hir::from(FunctionDef {
            name: Name::from("hello".to_string()),
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
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
