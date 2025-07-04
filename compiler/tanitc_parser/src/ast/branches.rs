use tanitc_ast::Ast;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_branch(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::KwLoop => self.parse_loop(),
            Lexem::KwWhile => self.parse_while(),
            Lexem::KwIf => self.parse_if(),
            Lexem::KwElse => self.parse_else(),
            _ => Err(Message::unexpected_token(
                next,
                &[Lexem::KwLoop, Lexem::KwWhile, Lexem::KwIf, Lexem::KwElse],
            )),
        }
    }

    fn parse_loop(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwLoop)?.location;

        let body = Box::new(self.parse_local_block()?);

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::Loop { body },
        }))
    }

    fn parse_while(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwWhile)?.location;

        let condition = Box::new(self.parse_expression()?);
        let body = Box::new(self.parse_local_block()?);

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::While { body, condition },
        }))
    }

    fn parse_if(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwIf)?.location;

        let condition = Box::new(self.parse_expression()?);
        let body = Box::new(self.parse_local_block()?);

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::If { condition, body },
        }))
    }

    fn parse_else(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::{Branch, BranchKind};

        let location = self.consume_token(Lexem::KwElse)?.location;

        let body = Box::new(if Lexem::KwIf == self.peek_token().lexem {
            self.parse_if()?
        } else {
            self.parse_local_block()?
        });

        Ok(Ast::from(Branch {
            location,
            kind: BranchKind::Else { body },
        }))
    }
}

#[test]
fn parse_loop_test() {
    use tanitc_ast::BranchKind;

    const SRC_TEXT: &str = "loop {\
                          \n   # come code here ...\
                          \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");
    let ast = parser.parse_branch().unwrap();

    let Ast::BranchStmt(br_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let BranchKind::Loop { body } = &br_node.kind else {
        panic!("Expected Loop, actually: {}", br_node.kind.to_str());
    };

    assert!(matches!(body.as_ref(), Ast::Block(_)));
}

#[test]
fn parse_while_test() {
    use tanitc_ast::{BranchKind, Value, ValueKind};

    const SRC_TEXT: &str = "while 1 {\
                          \n   # come code here ...\
                          \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");
    let ast = parser.parse_branch().unwrap();

    let Ast::BranchStmt(br_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let BranchKind::While { body, condition } = &br_node.kind else {
        panic!("Expected While, actually: {}", br_node.kind.to_str());
    };

    assert!(matches!(body.as_ref(), Ast::Block(_)));
    assert!(matches!(
        condition.as_ref(),
        Ast::Value(Value {
            kind: ValueKind::Integer(1),
            ..
        })
    ));
}

#[test]
fn parse_if_test() {
    use tanitc_ast::{BranchKind, Value, ValueKind};

    const SRC_TEXT: &str = "if 1 {\
                          \n   # come code here ...\
                          \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");
    let ast = parser.parse_branch().unwrap();

    let Ast::BranchStmt(br_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let BranchKind::If { body, condition } = &br_node.kind else {
        panic!("Expected If, actually: {}", br_node.kind.to_str());
    };

    assert!(matches!(
        condition.as_ref(),
        Ast::Value(Value {
            kind: ValueKind::Integer(1),
            ..
        })
    ));

    assert!(matches!(body.as_ref(), Ast::Block(_)));
}

#[test]
fn parse_if_else_test() {
    use tanitc_ast::{BranchKind, Value, ValueKind};

    const SRC_TEXT: &str = "if 1 { } else { }";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    // if
    {
        let ast = parser.parse_branch().unwrap();

        let Ast::BranchStmt(br_node) = &ast else {
            panic!("Expected BranchStmt, actually: {}", ast.name());
        };

        let BranchKind::If { body, condition } = &br_node.kind else {
            panic!("Expected If, actually: {}", br_node.kind.to_str());
        };

        assert!(matches!(
            condition.as_ref(),
            Ast::Value(Value {
                kind: ValueKind::Integer(1),
                ..
            })
        ));

        assert!(matches!(body.as_ref(), Ast::Block(_)));
    }

    // else
    {
        let ast = parser.parse_branch().unwrap();

        let Ast::BranchStmt(br_node) = &ast else {
            panic!("Expected BranchStmt, actually: {}", ast.name());
        };

        let BranchKind::Else { body } = &br_node.kind else {
            panic!("Expected Else, actually: {}", br_node.kind.to_str());
        };

        assert!(matches!(body.as_ref(), Ast::Block(_)));
    }
}

#[test]
fn parse_if_else_if_test() {
    use tanitc_ast::{BranchKind, Value, ValueKind};

    const SRC_TEXT: &str = "if 1 { }\
                          \nelse\
                          \nif 2 { }";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    // Parse if
    {
        let ast = parser.parse_branch().unwrap();

        let Ast::BranchStmt(br_node) = &ast else {
            panic!("Expected BranchStmt, actually: {}", ast.name());
        };

        let BranchKind::If { body, condition } = &br_node.kind else {
            panic!("Expected If, actually: {}", br_node.kind.to_str());
        };

        assert!(matches!(
            condition.as_ref(),
            Ast::Value(Value {
                kind: ValueKind::Integer(1),
                ..
            })
        ));

        assert!(matches!(body.as_ref(), Ast::Block(_)));
    }

    // else if
    {
        let ast = parser.parse_branch().unwrap();

        let Ast::BranchStmt(br_node) = &ast else {
            panic!("Expected BranchStmt, actually: {}", ast.name());
        };

        let BranchKind::Else { body } = &br_node.kind else {
            panic!("Expected Else, actually: {}", br_node.kind.to_str());
        };

        let Ast::BranchStmt(br_node) = body.as_ref() else {
            panic!("Expected BranchStmt, actually: {}", ast.name());
        };

        let BranchKind::If { body, condition } = &br_node.kind else {
            panic!("Expected Else, actually: {}", br_node.kind.to_str());
        };

        assert!(matches!(
            condition.as_ref(),
            Ast::Value(Value {
                kind: ValueKind::Integer(2),
                ..
            })
        ));

        assert!(matches!(body.as_ref(), Ast::Block(_)));
    }
}
