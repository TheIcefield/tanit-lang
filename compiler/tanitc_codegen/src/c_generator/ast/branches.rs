use tanitc_ast::ast::branches::{Branch, BranchKind};

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_branch(&mut self, branch: &Branch) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match &branch.kind {
            BranchKind::Loop { body } => {
                write!(self, "while (1)")?;

                self.generate(body)?;
            }
            BranchKind::While { body, condition } => {
                write!(self, "while (")?;

                self.generate(condition)?;

                writeln!(self, ")")?;

                self.generate(body)?;
            }
            BranchKind::If { condition, body } => {
                write!(self, "if (")?;
                self.generate(condition)?;
                writeln!(self, ")")?;

                self.generate(body)?;
            }
            BranchKind::Else { body } => {
                writeln!(self, "else")?;
                self.generate(body)?;
            }
        }
        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        blocks::Block,
        branches::{Branch, BranchKind},
        expressions::{BinaryOperation, Expression, ExpressionKind},
        values::{Value, ValueKind},
        Ast,
    };
    use tanitc_lexer::location::Location;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_cond() -> Expression {
        Expression {
            location: Location::default(),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::LogicalEq,
                lhs: Box::new(Ast::Value(Value {
                    kind: ValueKind::Integer(0),
                    location: Location::default(),
                })),
                rhs: Box::new(Ast::Value(Value {
                    kind: ValueKind::Integer(0),
                    location: Location::default(),
                })),
            },
        }
    }

    fn get_if() -> BranchKind {
        BranchKind::If {
            body: Box::new(Ast::from(Block::default())),
            condition: Box::new(get_cond().into()),
        }
    }

    fn get_else() -> BranchKind {
        BranchKind::Else {
            body: Box::new(Ast::from(Block::default())),
        }
    }

    fn get_else_if() -> BranchKind {
        BranchKind::Else {
            body: Box::new(Ast::from(Branch {
                location: Location::default(),
                kind: get_if(),
            })),
        }
    }

    #[test]
    fn codegen_if_test() {
        const SOURCE_EXPECTED: &str = "if (0 == 0)\
                                     \n{\
                                     \n}\n";

        let node = Ast::from(Branch {
            location: Location::default(),
            kind: get_if(),
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert!(header_res.is_empty());

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn codegen_if_else_test() {
        const SOURCE_EXPECTED: &str = "if (0 == 0)\
                                     \n{\
                                     \n}\
                                     \nelse\
                                     \n{\
                                     \n}\n";

        let node = Ast::from(Block {
            is_global: true,
            statements: vec![
                Branch {
                    location: Location::default(),
                    kind: get_if(),
                }
                .into(),
                Branch {
                    location: Location::default(),
                    kind: get_else(),
                }
                .into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert!(header_res.is_empty());

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn codegen_if_else_if_test() {
        const SOURCE_EXPECTED: &str = "if (0 == 0)\
                                     \n{\
                                     \n}\
                                     \nelse\
                                     \nif (0 == 0)\
                                     \n{\
                                     \n}\n";

        let node = Ast::from(Block {
            is_global: true,
            statements: vec![
                Branch {
                    location: Location::default(),
                    kind: get_if(),
                }
                .into(),
                Branch {
                    location: Location::default(),
                    kind: get_else_if(),
                }
                .into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert!(header_res.is_empty());

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn codegen_if_else_if_else_test() {
        const SOURCE_EXPECTED: &str = "if (0 == 0)\
                                     \n{\
                                     \n}\
                                     \nelse\
                                     \nif (0 == 0)\
                                     \n{\
                                     \n}\
                                     \nelse\
                                     \n{\
                                     \n}\n";

        let node = Ast::from(Block {
            is_global: true,
            statements: vec![
                Branch {
                    location: Location::default(),
                    kind: get_if(),
                }
                .into(),
                Branch {
                    location: Location::default(),
                    kind: get_else_if(),
                }
                .into(),
                Branch {
                    location: Location::default(),
                    kind: get_else(),
                }
                .into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert!(header_res.is_empty());

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
