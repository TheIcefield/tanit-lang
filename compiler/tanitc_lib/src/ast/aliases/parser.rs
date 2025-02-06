use super::AliasDef;
use crate::ast::{types::Type, Ast};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_parser::Parser;

impl AliasDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self {
            location: parser.consume_token(Lexem::KwAlias)?.location,
            identifier: parser.consume_identifier()?,
            ..Default::default()
        };

        parser.consume_token(Lexem::Assign)?;

        node.value = Type::parse(parser)?;

        Ok(Ast::from(node))
    }
}
