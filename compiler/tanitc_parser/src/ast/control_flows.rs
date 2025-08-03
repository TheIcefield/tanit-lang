use tanitc_ast::ast::{
    control_flows::{ControlFlow, ControlFlowKind},
    Ast,
};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_control_flow(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::KwBreak => self.parse_break(),
            Lexem::KwContinue => self.parse_continue(),
            Lexem::KwReturn => self.parse_return(),
            _ => Err(Message::unexpected_token(
                next,
                &[Lexem::KwBreak, Lexem::KwContinue, Lexem::KwReturn],
            )),
        }
    }

    fn parse_break(&mut self) -> Result<Ast, Message> {
        let location = self.consume_token(Lexem::KwBreak)?.location;

        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);

        let mut node = ControlFlow {
            location,
            kind: ControlFlowKind::Break { ret: None },
        };

        match self.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => {
                let expr = self.parse_expression()?;

                node.kind = ControlFlowKind::Break {
                    ret: Some(Box::new(expr)),
                }
            }
        }

        self.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    fn parse_continue(&mut self) -> Result<Ast, Message> {
        let location = self.consume_token(Lexem::KwContinue)?.location;

        Ok(Ast::from(ControlFlow {
            location,
            kind: ControlFlowKind::Continue,
        }))
    }

    fn parse_return(&mut self) -> Result<Ast, Message> {
        let location = self.consume_token(Lexem::KwReturn)?.location;

        let mut node = ControlFlow {
            location,
            kind: ControlFlowKind::Return { ret: None },
        };

        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);

        match self.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => {
                let expr = self.parse_expression()?;

                node.kind = ControlFlowKind::Return {
                    ret: Some(Box::new(expr)),
                }
            }
        }

        self.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }
}

#[test]
fn parse_return_value_test() {
    use tanitc_ast::ast::values::{Value, ValueKind};

    const SRC_TEXT: &str = "return 10\n";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();
    let ast = parser.parse_control_flow().unwrap();

    let Ast::ControlFlow(cf_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let ControlFlowKind::Return { ret } = &cf_node.kind else {
        panic!("Expected Return, actually: {}", cf_node.kind.to_str());
    };

    let Ast::Value(Value {
        kind: ValueKind::Integer(10),
        ..
    }) = ret.as_ref().unwrap().as_ref()
    else {
        panic!("Expected ValueKind::Integer(10), actually: {ret:?}");
    };
}

#[test]
fn parse_return_test() {
    const SRC_TEXT: &str = "return\n";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();
    let ast = parser.parse_control_flow().unwrap();

    let Ast::ControlFlow(cf_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let ControlFlowKind::Return { ret } = &cf_node.kind else {
        panic!("Expected Return, actually: {}", cf_node.kind.to_str());
    };

    assert!(ret.is_none());
}

#[test]
fn parse_break_test() {
    const SRC_TEXT: &str = "break\n";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();
    let ast = parser.parse_control_flow().unwrap();

    let Ast::ControlFlow(cf_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let ControlFlowKind::Break { ret } = &cf_node.kind else {
        panic!("Expected Return, actually: {}", cf_node.kind.to_str());
    };

    assert!(ret.is_none());
}

#[test]
fn parse_continue_test() {
    const SRC_TEXT: &str = "continue\n";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();
    let ast = parser.parse_control_flow().unwrap();

    let Ast::ControlFlow(cf_node) = &ast else {
        panic!("Expected ControlFlow, actually: {}", ast.name());
    };

    let ControlFlowKind::Continue = &cf_node.kind else {
        panic!("Expected Continue, actually: {}", cf_node.kind.to_str());
    };
}
