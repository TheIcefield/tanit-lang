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

    use tanitc_hir::hir::{blocks::Block, type_spec::Type, Hir};
    use tanitc_hir_test::{create_func_def, create_program};

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
        // Given
        let node = create_program(vec![create_func_def(
            "hello",
            vec![],
            Type::unit(),
            vec![
                get_block(false, vec![]).into(),
                get_block(false, vec![]).into(),
            ],
        )
        .into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        node.accept(&mut writer).unwrap();

        // Then
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

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
