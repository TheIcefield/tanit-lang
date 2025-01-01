use super::FunctionDef;
use crate::ast::{
    identifiers::Identifier, scopes::Scope, types::Type, variables::VariableDef, Ast,
};
use crate::messages::Message;
use crate::parser::{token::Lexem, Parser};

impl FunctionDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::parse_header(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            Lexem::Lcb => {
                node.body = Some(Box::new(Scope::parse_local(parser)?));
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::Lcb, Lexem::EndOfLine],
                ));
            }
        }

        Ok(Ast::FuncDef { node })
    }

    fn parse_header(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.consume_token(Lexem::KwFunc)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        let parameters = Self::parse_header_params(parser)?;

        let next = parser.peek_token();
        let return_type = if next.lexem == Lexem::Arrow {
            parser.get_token();
            Type::parse(parser)?
        } else {
            Type::unit()
        };

        Ok(Self {
            location,
            identifier,
            return_type,
            parameters,
            body: None,
        })
    }

    fn parse_header_params(parser: &mut Parser) -> Result<Vec<Ast>, Message> {
        parser.consume_token(Lexem::LParen)?;

        let mut parameters = Vec::<Ast>::new();
        loop {
            let next = parser.peek_token();

            if next.is_identifier() {
                parameters.push(Ast::VariableDef {
                    node: Self::parse_param(parser)?,
                });

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

        Ok(parameters)
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
