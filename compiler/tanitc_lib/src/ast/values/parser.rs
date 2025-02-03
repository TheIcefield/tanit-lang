use super::{CallParam, Value, ValueType};
use crate::ast::{expressions::Expression, identifiers::Identifier, Ast};
use crate::messages::Message;
use crate::parser::Parser;

use tanitc_lexer::token::Lexem;

impl Value {
    pub fn parse_call_params(parser: &mut Parser) -> Result<Vec<CallParam>, Message> {
        let _ = parser.consume_token(Lexem::LParen)?.location;

        let mut args = Vec::<CallParam>::new();

        let mut i = 0;
        loop {
            let next = parser.peek_token();

            if next.lexem == Lexem::RParen {
                break;
            }

            let expr = Expression::parse(parser)?;

            let param_id = if let Ast::Value(Value {
                location: _,
                value: ValueType::Identifier(id),
            }) = &expr
            {
                if parser.peek_token().lexem == Lexem::Colon {
                    parser.consume_token(Lexem::Colon)?;
                    Some(id.clone())
                } else {
                    None
                }
            } else {
                None
            };

            let param = if let Some(id) = param_id {
                CallParam::Notified(id, Box::new(Expression::parse(parser)?))
            } else {
                CallParam::Positional(i, Box::new(expr))
            };

            args.push(param);

            i += 1;

            let next = parser.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == Lexem::RParen {
                // end parsing if ')'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        parser.consume_token(Lexem::RParen)?;

        Ok(args)
    }

    pub fn parse_array(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::Lsb)?.location;

        let mut components = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            if next.lexem == Lexem::Rsb {
                break;
            }
            components.push(Expression::parse(parser)?);

            let next = parser.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == Lexem::Rsb {
                // end parsing if ']'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        parser.consume_token(Lexem::Rsb)?;

        Ok(Ast::from(Self {
            location,
            value: ValueType::Array { components },
        }))
    }

    pub fn parse_struct(parser: &mut Parser) -> Result<Vec<(Identifier, Ast)>, Message> {
        parser.consume_token(Lexem::Lcb)?;

        let mut components = Vec::<(Identifier, Ast)>::new();

        loop {
            let next = parser.peek_token();

            if next.lexem == Lexem::Rcb {
                break;
            }

            let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

            parser.consume_token(Lexem::Colon)?;

            components.push((identifier, Expression::parse(parser)?));

            let next = parser.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == Lexem::Rcb {
                // end parsing if '}'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        parser.consume_token(Lexem::Rcb)?;

        Ok(components)
    }
}
