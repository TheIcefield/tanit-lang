use super::{VariantDef, VariantField};
use crate::ast::{identifiers::Identifier, structs::StructDef, types::Type, Ast};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_parser::Parser;

impl VariantField {
    pub fn parse(parser: &mut Parser) -> Result<Self, Message> {
        let mut node = Self::default();

        let old_opt = parser.does_ignore_nl();
        parser.set_ignore_nl_option(false);

        node.parse_internal(parser)?;

        parser.set_ignore_nl_option(old_opt);

        Ok(node)
    }

    fn parse_internal(&mut self, parser: &mut Parser) -> Result<(), Message> {
        let next = parser.peek_token();
        match next.lexem {
            Lexem::EndOfLine => {
                *self = VariantField::Common;

                Ok(())
            }

            Lexem::LParen => {
                if let Type::Tuple { components } = Type::parse_tuple_def(parser)? {
                    *self = Self::TupleLike(components);

                    Ok(())
                } else {
                    Err(Message::unexpected_token(next, &[]))
                }
            }

            Lexem::Lcb => {
                let mut node = StructDef::default();
                node.parse_body(parser)?;

                *self = VariantField::StructLike(node.fields);

                Ok(())
            }

            _ => Err(Message::new(
                next.location,
                &format!("Unexpected token during parsing enum: {}", next),
            )),
        }
    }
}

impl VariantDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_body(parser)?;

        Ok(Ast::from(node))
    }

    fn parse_header(&mut self, parser: &mut Parser) -> Result<(), Message> {
        self.location = parser.consume_token(Lexem::KwVariant)?.location;
        self.identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(())
    }

    fn parse_body(&mut self, parser: &mut Parser) -> Result<(), Message> {
        parser.consume_token(Lexem::Lcb)?;
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        self.parse_body_internal(parser)?;
        parser.set_ignore_nl_option(old_opt);

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

                Lexem::KwStruct => self.internals.push(StructDef::parse(parser)?),

                Lexem::KwVariant => self.internals.push(VariantDef::parse(parser)?),

                Lexem::Identifier(id) => {
                    let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                    if self.fields.contains_key(&identifier) {
                        parser.error(Message::new(
                            next.location,
                            &format!("Enum has already field with identifier \"{}\"", id),
                        ));
                        continue;
                    }

                    self.fields.insert(identifier, VariantField::parse(parser)?);

                    parser.consume_new_line()?;
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        &format!(
                            "{}\nHelp: {}{}",
                            "Unexpected token: \"{\" during parsing enum fields.",
                            "If you tried to declare struct-like field, place \"{\" ",
                            "in the same line with name of the field."
                        ),
                    ));
                }

                _ => {
                    return Err(Message::unexpected_token(next, &[]));
                }
            }
        }

        Ok(())
    }
}
