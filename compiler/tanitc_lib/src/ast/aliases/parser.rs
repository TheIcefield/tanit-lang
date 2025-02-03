use super::AliasDef;
use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::parser::Parser;

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

impl AliasDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let location = parser.consume_token(Lexem::KwAlias)?.location;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        parser.consume_token(Lexem::Assign)?;

        let value = Type::parse(parser)?;

        Ok(Ast::from(Self {
            location,
            identifier,
            value,
        }))
    }
}
