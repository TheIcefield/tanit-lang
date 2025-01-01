use super::{Branch, BranchType, Break, Continue, Return};
use crate::ast::{expressions::Expression, scopes::Scope, Ast};
use crate::messages::Message;
use crate::parser::{token::Lexem, Parser};

impl Branch {
    pub fn parse_loop(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwLoop)?.location;

        let body = Box::new(Scope::parse_local(parser)?);

        Ok(Ast::BranchStmt {
            node: Self {
                location,
                branch: BranchType::Loop {
                    body,
                    condition: None,
                },
            },
        })
    }

    pub fn parse_while(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwWhile)?.location;

        let condition = Some(Box::new(Expression::parse(parser)?));

        let body = Box::new(Scope::parse_local(parser)?);

        Ok(Ast::BranchStmt {
            node: Self {
                location,
                branch: BranchType::Loop { body, condition },
            },
        })
    }

    pub fn parse_if(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwIf)?.location;

        let condition = Box::new(Expression::parse(parser)?);

        let main_body = Box::new(Scope::parse_local(parser)?);

        let else_body = if parser.peek_token().lexem == Lexem::KwElse {
            parser.get_token();

            let next = parser.peek_token();
            match next.lexem {
                Lexem::KwIf => Some(Box::new(Self::parse_if(parser)?)),
                Lexem::Lcb => Some(Box::new(Scope::parse_local(parser)?)),
                _ => {
                    parser.error(Message::unexpected_token(next, &[Lexem::KwIf, Lexem::Lcb]));
                    None
                }
            }
        } else {
            None
        };

        Ok(Ast::BranchStmt {
            node: Self {
                location,
                branch: BranchType::IfElse {
                    condition,
                    main_body,
                    else_body,
                },
            },
        })
    }
}

impl Break {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwBreak)?.location;

        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);

        let mut node = Break {
            location,
            expr: None,
        };

        match parser.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => node.expr = Some(Box::new(Expression::parse(parser)?)),
        }

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::BreakStmt { node })
    }
}

impl Continue {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwContinue)?.location;

        Ok(Ast::ContinueStmt {
            node: Self { location },
        })
    }
}

impl Return {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwReturn)?.location;

        let mut node = Return {
            location,
            expr: None,
        };

        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);

        match parser.peek_token().lexem {
            Lexem::EndOfLine => {}
            _ => node.expr = Some(Box::new(Expression::parse(parser)?)),
        }

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::ReturnStmt { node })
    }
}
