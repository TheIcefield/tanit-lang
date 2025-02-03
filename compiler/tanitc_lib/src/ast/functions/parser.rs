use super::FunctionDef;
use crate::ast::{
    identifiers::Identifier, scopes::Scope, types::Type, variables::VariableDef, Ast,
};
use crate::parser::Parser;

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

impl FunctionDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_body(parser)?;

        Ok(Ast::from(node))
    }

    fn parse_header(&mut self, parser: &mut Parser) -> Result<(), Message> {
        self.location = parser.consume_token(Lexem::KwFunc)?.location;

        self.identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        self.parse_header_params(parser)?;

        self.return_type = if Lexem::Arrow == parser.peek_token().lexem {
            parser.get_token();
            Type::parse(parser)?
        } else {
            Type::unit()
        };

        Ok(())
    }

    fn parse_header_params(&mut self, parser: &mut Parser) -> Result<(), Message> {
        parser.consume_token(Lexem::LParen)?;

        loop {
            let next = parser.peek_token();

            if next.is_identifier() {
                self.parameters
                    .push(Ast::VariableDef(Self::parse_param(parser)?));

                let next = parser.peek_token();
                if next.lexem == Lexem::Comma {
                    parser.get_token();
                } else if next.lexem == Lexem::RParen {
                    continue;
                } else {
                    return Err(Message::unexpected_token(next, &[]));
                }
            } else if next.lexem == Lexem::RParen {
                parser.get_token();
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        Ok(())
    }

    fn parse_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        let next = parser.peek_token();
        match next.lexem {
            Lexem::Lcb => {
                self.body = Some(Box::new(Scope::parse_local(parser)?));
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::Lcb, Lexem::EndOfLine],
                ));
            }
        }

        Ok(())
    }

    /* parse function param */
    fn parse_param(parser: &mut Parser) -> Result<VariableDef, Message> {
        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        parser.consume_token(Lexem::Colon)?;

        let var_type = Type::parse(parser)?;

        Ok(VariableDef {
            location: identifier.location,
            identifier,
            var_type,
            is_global: false,
            is_mutable: true,
        })
    }
}
