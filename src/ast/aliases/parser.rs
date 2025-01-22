use super::AliasDef;
use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::messages::Message;
use crate::parser::{token::Lexem, Parser};

impl AliasDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwAlias)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        parser.consume_token(Lexem::Assign)?;

        let value = Type::parse(parser)?;

        Ok(Ast::AliasDef {
            node: AliasDef {
                location,
                identifier,
                value,
            },
        })
    }
}
