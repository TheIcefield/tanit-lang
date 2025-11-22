use tanitc_ast::attributes::ParsedAttributes;
use tanitc_attributes::{Publicity, Safety};
use tanitc_lexer::token::{Lexem, Token};
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_attributes(&mut self) -> Result<ParsedAttributes, Message> {
        let mut attrs = ParsedAttributes::default();

        loop {
            let Some(next) = self.peek_token() else {
                break;
            };

            let res = match next.lexem_ref() {
                Lexem::KwSafe | Lexem::KwUnsafe => {
                    self.get_token();
                    self.set_safety(&mut attrs, &next)
                }

                Lexem::KwPub => {
                    self.get_token();
                    self.set_publicity(&mut attrs, &next)
                }
                _ => break,
            };

            if let Err(err) = res {
                self.error(err);
            }
        }

        Ok(attrs)
    }

    fn set_safety(&self, attrs: &mut ParsedAttributes, tkn: &Token) -> Result<(), Message> {
        let safety = match tkn.lexem_ref() {
            Lexem::KwSafe => Safety::Safe,
            Lexem::KwUnsafe => Safety::Unsafe,
            _ => {
                return Err(Message::unexpected_token(
                    tkn,
                    &[Lexem::KwSafe, Lexem::KwUnsafe],
                ))
            }
        };

        if attrs.safety.is_some() {
            return Err(Message::from_string(
                tkn.location_ref(),
                format!(
                    "Setting \"{safety}\" discards previous setting: \"{}\"",
                    attrs.safety.unwrap()
                ),
            ));
        }

        attrs.safety = Some(safety);

        Ok(())
    }

    fn set_publicity(&self, attrs: &mut ParsedAttributes, tkn: &Token) -> Result<(), Message> {
        let Lexem::KwPub = tkn.lexem_ref() else {
            return Err(Message::unexpected_token(tkn, &[Lexem::KwPub]));
        };

        if attrs.publicity.is_some() {
            return Err(Message::from_string(
                tkn.location_ref(),
                "Impossible to set publicity more than one time".to_string(),
            ));
        }

        attrs.publicity = Some(Publicity::Public);

        Ok(())
    }
}

#[test]
fn attrs_test() {
    const SRC_TEXT: &str = "unsafe pub";

    let mut parser = Parser::from_text(SRC_TEXT);
    let attrs = parser.parse_attributes().unwrap();

    assert_eq!(attrs.publicity, Some(Publicity::Public));
    assert_eq!(attrs.safety, Some(Safety::Unsafe));
}

#[test]
fn attrs_pub_test() {
    const SRC_TEXT: &str = "pub";

    let mut parser = Parser::from_text(SRC_TEXT);
    let attrs = parser.parse_attributes().unwrap();

    assert_eq!(attrs.publicity, Some(Publicity::Public));
    assert_eq!(attrs.safety, None);
}

#[test]
fn attrs_safe_test() {
    const SRC_TEXT: &str = "safe";

    let mut parser = Parser::from_text(SRC_TEXT);
    let attrs = parser.parse_attributes().unwrap();

    assert_eq!(attrs.publicity, None);
    assert_eq!(attrs.safety, Some(Safety::Safe));
}

#[test]
fn attrs_incorrect_test() {
    const SRC_TEXT: &str = "unsafe pub safe pub";

    let mut parser = Parser::from_text(SRC_TEXT);
    let _ = parser.parse_attributes().unwrap();

    let errors = parser.get_errors();

    assert_eq!(errors.len(), 2);
    assert_eq!(
        errors[0].text,
        "Syntax error: Setting \"Safe\" discards previous setting: \"Unsafe\""
    );
    assert_eq!(
        errors[1].text,
        "Syntax error: Impossible to set publicity more than one time"
    );
}
