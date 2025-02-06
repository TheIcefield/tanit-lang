use super::StructDef;
use crate::ast::{types::Type, variants::VariantDef, Ast};
use tanitc_parser::Parser;

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

impl StructDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_body(parser)?;

        Ok(Ast::from(node))
    }

    fn parse_header(&mut self, parser: &mut Parser) -> Result<(), Message> {
        self.location = parser.consume_token(Lexem::KwStruct)?.location;
        self.identifier = parser.consume_identifier()?;
        Ok(())
    }

    pub fn parse_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        parser.consume_token(Lexem::Lcb)?;

        self.parse_body_internal(parser)?;

        parser.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_body_internal(&mut self, parser: &mut Parser) -> Result<(), Message> {
        loop {
            let next = parser.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwStruct => {
                    self.internals.push(StructDef::parse(parser)?);
                }

                Lexem::KwVariant => {
                    self.internals.push(VariantDef::parse(parser)?);
                }

                Lexem::Identifier(id) => {
                    let identifier = parser.consume_identifier()?;

                    if self.fields.contains_key(&identifier) {
                        parser.error(Message::new(
                            next.location,
                            &format!("Struct has already field with identifier {}", id),
                        ));
                        continue;
                    }

                    parser.consume_token(Lexem::Colon)?;

                    self.fields.insert(identifier, Type::parse(parser)?);
                }

                _ => {
                    return Err(Message::new(
                        next.location,
                        "Unexpected token when parsing struct fields",
                    ));
                }
            }
        }

        Ok(())
    }
}
