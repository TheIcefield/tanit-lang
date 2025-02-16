use super::{Branch, BranchType, Interupter, InterupterType};
use crate::ast::{expressions::Expression, scopes::Scope, Ast};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_parser::Parser;

impl Branch {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let next = parser.peek_token();
        match next.lexem {
            Lexem::KwLoop => Self::parse_loop(parser),
            Lexem::KwWhile => Self::parse_while(parser),
            Lexem::KwIf => Self::parse_if(parser),
            Lexem::KwElse => Self::parse_else(parser),
            _ => Err(Message::unexpected_token(
                next,
                &[Lexem::KwLoop, Lexem::KwWhile, Lexem::KwIf, Lexem::KwElse],
            )),
        }
    }

    pub fn parse_loop(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwLoop)?.location;

        let body = Self::parse_body(parser)?;

        Ok(Ast::from(Self {
            location,
            branch: BranchType::Loop { body },
        }))
    }

    pub fn parse_while(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwWhile)?.location;

        let condition = Self::parse_condition(parser)?;
        let body = Self::parse_body(parser)?;

        Ok(Ast::from(Self {
            location,
            branch: BranchType::While { body, condition },
        }))
    }

    pub fn parse_if(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwIf)?.location;

        let condition = Self::parse_condition(parser)?;
        let body = Self::parse_body(parser)?;

        Ok(Ast::from(Self {
            location,
            branch: BranchType::If { condition, body },
        }))
    }

    pub fn parse_else(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwElse)?.location;

        let body = if Lexem::KwIf == parser.peek_token().lexem {
            Box::new(Self::parse_if(parser)?)
        } else {
            Self::parse_body(parser)?
        };

        Ok(Ast::from(Branch {
            location,
            branch: BranchType::Else { body },
        }))
    }

    fn parse_condition(parser: &mut Parser) -> Result<Box<Ast>, Message> {
        Ok(Box::new(Expression::parse(parser)?))
    }

    fn parse_body(parser: &mut Parser) -> Result<Box<Ast>, Message> {
        Ok(Box::new(Scope::parse_local(parser)?))
    }
}

impl Interupter {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let next = parser.peek_token();
        match next.lexem {
            Lexem::KwBreak => Self::parse_break(parser),
            Lexem::KwContinue => Self::parse_continue(parser),
            Lexem::KwReturn => Self::parse_return(parser),
            _ => Err(Message::unexpected_token(
                next,
                &[Lexem::KwBreak, Lexem::KwContinue, Lexem::KwReturn],
            )),
        }
    }

    pub fn parse_break(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwBreak)?.location;

        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);

        let mut node = Self {
            location,
            interupter: InterupterType::Break { ret: None },
        };

        match parser.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => {
                node.interupter = InterupterType::Break {
                    ret: Some(node.parse_ret_expr(parser)?),
                }
            }
        }

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    pub fn parse_continue(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwContinue)?.location;

        Ok(Ast::from(Self {
            location,
            interupter: InterupterType::Continue,
        }))
    }

    pub fn parse_return(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwReturn)?.location;

        let mut node = Self {
            location,
            interupter: InterupterType::Return { ret: None },
        };

        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);

        match parser.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => {
                node.interupter = InterupterType::Return {
                    ret: Some(node.parse_ret_expr(parser)?),
                }
            }
        }

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    fn parse_ret_expr(&mut self, parser: &mut Parser) -> Result<Expression, Message> {
        if let Ast::Expression(node) = Expression::parse(parser)? {
            Ok(node)
        } else {
            Err(Message::unreachable(self.location))
        }
    }
}
