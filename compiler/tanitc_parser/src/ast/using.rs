use tanitc_ast::ast::{
    uses::{Use, UseIdentifier},
    Ast,
};
use tanitc_ident::Ident;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_use(&mut self) -> Result<Ast, Message> {
        let mut u = Use::default();

        self.parse_use_internal(&mut u)?;

        Ok(Ast::from(u))
    }

    fn parse_use_internal(&mut self, u: &mut Use) -> Result<(), Message> {
        u.location = self.consume_token(Lexem::KwUse)?.location_ref().clone();

        loop {
            let Some(next) = self.peek_token() else {
                break;
            };
            let id = match next.lexem_ref() {
                Lexem::KwSuper => {
                    self.get_token();
                    UseIdentifier::BuiltInSuper
                }
                Lexem::KwSelfT => {
                    self.get_token();
                    UseIdentifier::BuiltInSelf
                }
                Lexem::KwCrate => {
                    self.get_token();
                    UseIdentifier::BuiltInCrate
                }
                Lexem::Star => {
                    self.get_token();

                    if self.is_eof() {
                        UseIdentifier::BuiltInAll
                    } else if self.is_next(Lexem::EndOfLine) {
                        self.get_token();
                        UseIdentifier::BuiltInAll
                    } else {
                        return Err(Message::unexpected_token(
                            self.peek_token().as_ref().unwrap(),
                            &[Lexem::EndOfLine],
                        ));
                    }
                }
                Lexem::Identifier(id) => {
                    self.get_token();
                    UseIdentifier::Identifier(Ident::from(id.clone()))
                }
                _ => {
                    return Err(Message::unexpected_token(
                        &next,
                        &[
                            Lexem::KwSuper,
                            Lexem::KwSelfT,
                            Lexem::KwCrate,
                            Lexem::Identifier("".into()),
                        ],
                    ))
                }
            };

            u.identifier.push(id);

            let Some(next) = self.peek_token() else {
                break;
            };
            match next.lexem_ref() {
                Lexem::EndOfLine => {
                    self.get_token();
                    break;
                }
                Lexem::Dcolon => {
                    self.get_token();
                }
                _ => {
                    return Err(Message::unexpected_token(
                        &next,
                        &[Lexem::Dcolon, Lexem::EndOfLine],
                    ))
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_test() {
        const SRC_TEXT: &str = "use hello::world";

        let hello_id = Ident::from("hello".to_string());
        let world_id = Ident::from("world".to_string());

        let mut parser = Parser::from_text(SRC_TEXT);

        let use_node = parser.parse_use().unwrap();
        {
            if parser.has_errors() {
                panic!("{:?}", parser.get_errors());
            }
        }

        let Ast::Use(u) = use_node else {
            panic!("Expected Ast::Use, actually: {}", use_node.name());
        };

        assert_eq!(
            u.identifier,
            [
                UseIdentifier::Identifier(hello_id),
                UseIdentifier::Identifier(world_id),
            ]
        );
    }

    #[test]
    fn parse_use_all_test() {
        const SRC_TEXT: &str = "use crate::mod::*";

        let mod_id = Ident::from("mod".to_string());

        let mut parser = Parser::from_text(SRC_TEXT);

        let use_node = parser.parse_use().unwrap();
        {
            if parser.has_errors() {
                panic!("{:?}", parser.get_errors());
            }
        }

        let Ast::Use(u) = use_node else {
            panic!("Expected Ast::Use, actually: {}", use_node.name());
        };

        assert_eq!(
            u.identifier,
            [
                UseIdentifier::BuiltInCrate,
                UseIdentifier::Identifier(mod_id),
                UseIdentifier::BuiltInAll
            ]
        );
    }

    #[test]
    fn use_all_wrong_test() {
        const SRC_TEXT: &str = "use Self::mod::*::hi";

        let mut parser = Parser::from_text(SRC_TEXT);

        parser
            .parse_use()
            .err()
            .expect("Expected fail on parse_use");
    }
}
