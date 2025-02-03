use super::EnumDef;
use crate::ast::{identifiers::Identifier, Ast};
use crate::messages::Message;
use crate::parser::Parser;

use tanitc_lexer::token::Lexem;

impl EnumDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_header(parser)?;
        node.parse_body(parser)?;

        Ok(Ast::from(node))
    }

    fn parse_header(&mut self, parser: &mut Parser) -> Result<(), Message> {
        self.location = parser.consume_token(Lexem::KwEnum)?.location;
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
                Lexem::Identifier(id) => {
                    let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                    let value = if Lexem::Colon == parser.peek_token().lexem {
                        parser.consume_token(Lexem::Colon)?;

                        let token = parser.consume_integer()?;
                        let value = if let Lexem::Integer(value) = token.lexem {
                            match value.parse::<usize>() {
                                Ok(value) => value,
                                Err(err) => {
                                    return Err(Message::parse_int_error(token.location, err))
                                }
                            }
                        } else {
                            unreachable!()
                        };

                        Some(value)
                    } else {
                        None
                    };

                    if self.fields.contains_key(&identifier) {
                        parser.error(Message::new(
                            next.location,
                            &format!("Enum has already field with identifier \"{}\"", id),
                        ));
                        continue;
                    }

                    self.fields.insert(identifier, value);

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
