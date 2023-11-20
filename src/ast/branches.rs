use crate::ast::{expressions, scopes, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct LoopNode {
    body: scopes::Scope,
    condition: Option<Box<Ast>>,
}

#[derive(Clone)]
pub struct Break {
    pub expr: Option<Box<Ast>>,
}

#[derive(Clone)]
pub struct Continue {
    pub expr: Option<Box<Ast>>,
}

#[derive(Clone)]
pub struct Return {
    pub expr: Option<Box<Ast>>,
}

impl IAst for LoopNode {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<loop>", put_intent(intent))?;

        if let Some(cond) = &self.condition {
            writeln!(stream, "{}<condition>", put_intent(intent + 1))?;

            cond.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</condition>", put_intent(intent + 1))?;
        }

        self.body.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</loop>", put_intent(intent))?;

        Ok(())
    }
}

impl IAst for Break {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match &self.expr {
            Some(cond) => {
                writeln!(stream, "{}<break>", put_intent(intent))?;
                cond.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</break>", put_intent(intent))?;
            }
            _ => {
                writeln!(stream, "{}<break/>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}

impl IAst for Continue {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match &self.expr {
            Some(cond) => {
                writeln!(stream, "{}<continue>", put_intent(intent))?;
                cond.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</continue>", put_intent(intent))?;
            }
            _ => {
                writeln!(stream, "{}<continue/>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}

impl IAst for Return {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match &self.expr {
            Some(cond) => {
                writeln!(stream, "{}<return>", put_intent(intent))?;
                cond.traverse(stream, intent + 1)?;
                writeln!(stream, "{}</return>", put_intent(intent))?;
            }
            _ => {
                writeln!(stream, "{}<return/>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}

pub fn parse_loop(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwLoop)?;

    let body = scopes::parse_local_external(parser)?;

    Some(Ast::LoopStmt {
        node: LoopNode {
            body,
            condition: None,
        },
    })
}

pub fn parse_while(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwWhile)?;

    let condition = parse_condition(parser)?;

    let body = scopes::parse_local_external(parser)?;

    Some(Ast::LoopStmt {
        node: LoopNode {
            body,
            condition: Some(Box::new(condition)),
        },
    })
}

pub fn parse_break(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwBreak)?;

    let mut node = Break { expr: None };

    match parser.peek_token().lexem {
        TokenType::EndOfLine => {}
        _ => node.expr = Some(Box::new(expressions::parse_expression(parser)?)),
    }

    Some(Ast::BreakStmt { node })
}

pub fn parse_continue(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwContinue)?;

    let mut node = Continue { expr: None };

    match parser.peek_token().lexem {
        TokenType::EndOfLine => {}
        _ => node.expr = Some(Box::new(expressions::parse_expression(parser)?)),
    }

    Some(Ast::ContinueStmt { node })
}

pub fn parse_return(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwReturn)?;

    let mut node = Return { expr: None };

    match parser.peek_token().lexem {
        TokenType::EndOfLine => {}
        _ => node.expr = Some(Box::new(expressions::parse_expression(parser)?)),
    }

    Some(Ast::ReturnStmt { node })
}

pub fn parse_condition(parser: &mut Parser) -> Option<Ast> {
    expressions::parse_expression(parser)
}
