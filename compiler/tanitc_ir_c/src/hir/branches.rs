use tanitc_hir::hir::branches::{Branch, Else, ElseBody, If, Loop, While};

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_branch(&mut self, branch: &Branch) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        match branch {
            Branch::Loop(node) => self.generate_loop(node)?,
            Branch::While(node) => self.generate_while(node)?,
            Branch::If(node) => self.generate_if(node)?,
            Branch::Else(node) => self.generate_else(node)?,
        }
        self.mode = old_mode;
        Ok(())
    }

    fn generate_loop(&mut self, branch: &Loop) -> std::io::Result<()> {
        use std::io::Write;

        write!(self, "while (1)")?;
        self.generate_block(&branch.body)?;

        Ok(())
    }

    fn generate_while(&mut self, branch: &While) -> std::io::Result<()> {
        use std::io::Write;

        write!(self, "while (")?;
        self.generate_expression(&branch.condition)?;
        writeln!(self, ")")?;
        self.generate_block(&branch.body)?;

        Ok(())
    }

    fn generate_if(&mut self, branch: &If) -> std::io::Result<()> {
        use std::io::Write;

        write!(self, "if (")?;
        self.generate_expression(&branch.condition)?;
        writeln!(self, ")")?;
        self.generate_block(&branch.body)?;

        Ok(())
    }

    fn generate_else(&mut self, branch: &Else) -> std::io::Result<()> {
        use std::io::Write;

        writeln!(self, "else")?;

        match &branch.body {
            ElseBody::Block(node) => self.generate_block(node),
            ElseBody::If(node) => self.generate_if(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_hir::hir::{
        blocks::Block,
        branches::{Branch, Else, ElseBody, If},
        expressions::{
            binary::{BinaryExpr, BinaryOperation},
            literal, Expression,
        },
        Hir,
    };
    use tanitc_lexer::location::Location;

    use pretty_assertions::assert_str_eq;

    fn get_cond() -> Expression {
        Expression::Binary(BinaryExpr {
            location: Location::default(),
            operation: BinaryOperation::LogicalEq,
            lhs: Box::new(Expression::Literal(literal::Literal::Integer(
                literal::Integer {
                    location: Location::default(),
                    value: 0,
                },
            ))),
            rhs: Box::new(Expression::Literal(literal::Literal::Integer(
                literal::Integer {
                    location: Location::default(),
                    value: 0,
                },
            ))),
        })
    }

    fn get_if() -> If {
        If {
            location: Location::default(),
            body: Box::new(Block::default()),
            condition: Box::new(get_cond().into()),
        }
    }

    fn get_else() -> Else {
        Else {
            location: Location::default(),
            body: ElseBody::Block(Box::new(Block::default())),
        }
    }

    fn get_else_if() -> Else {
        Else {
            location: Location::default(),
            body: ElseBody::If(Box::new(get_if())),
        }
    }

    #[test]
    fn codegen_if_test() {
        const SOURCE_EXPECTED: &str = "if (0 == 0)\
                                     \n{\
                                     \n}\n";

        let node = Hir::from(Branch::If(get_if()));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

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

        let node = Hir::from(Block {
            is_global: true,
            statements: vec![Branch::If(get_if()).into(), Branch::Else(get_else()).into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

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

        let node = Hir::from(Block {
            is_global: true,
            statements: vec![
                Branch::If(get_if()).into(),
                Branch::Else(get_else_if()).into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

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

        let node = Hir::from(Block {
            is_global: true,
            statements: vec![
                Branch::If(get_if()).into(),
                Branch::Else(get_else_if()).into(),
                Branch::Else(get_else()).into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert!(header_res.is_empty());

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
