use crate::ast::{expressions, scopes, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Branch {
    condition: Box<Ast>,
    main_body: Box<Ast>,
    else_body: Option<Box<Ast>>,
}

#[derive(Clone)]
pub struct LoopNode {
    body: Box<Ast>,
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

impl IAst for Branch {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<branch>", put_intent(intent))?;

        {
            writeln!(stream, "{}<condition>", put_intent(intent + 1))?;

            self.condition.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</condition>", put_intent(intent + 1))?;
        }

        {
            writeln!(stream, "{}<true>", put_intent(intent + 1))?;

            self.main_body.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</true>", put_intent(intent + 1))?;
        }

        if let Some(else_body) = &self.else_body {
            writeln!(stream, "{}<false>", put_intent(intent + 1))?;

            else_body.traverse(stream, intent + 2)?;

            writeln!(stream, "{}</false>", put_intent(intent + 1))?;
        }

        writeln!(stream, "{}</branch>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse_loop(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwLoop)?;

    let body = Box::new(scopes::parse_local_external(parser)?);

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

    let body = Box::new(scopes::parse_local_external(parser)?);

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

pub fn parse_if(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwIf)?;

    let condition = Box::new(parse_condition(parser)?);

    let main_body = Box::new(scopes::parse_local_external(parser)?);

    let else_body = if parser.peek_token().lexem == TokenType::KwElse {
        parser.get_token();

        let next = parser.peek_token();
        match next.lexem {
            TokenType::KwIf => Some(Box::new(parse_if(parser)?)),
            TokenType::Lcb => Some(Box::new(scopes::parse_local_external(parser)?)),
            _ => {
                parser.error(
                    &format!("Unexpected token \"{}\" in branch expression", next),
                    next.get_location(),
                );
                None
            }
        }
    } else {
        None
    };

    Some(Ast::IfStmt {
        node: Branch {
            condition,
            main_body,
            else_body,
        },
    })
}

pub fn parse_condition(parser: &mut Parser) -> Option<Ast> {
    expressions::parse_expression(parser)
}
