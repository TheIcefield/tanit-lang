use tanitc_ast::{Ast, Use, UseIdentifier};
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
        u.location = self.consume_token(Lexem::KwUse)?.location;

        loop {
            let next = self.peek_token();
            let id = match next.lexem {
                Lexem::KwSuper => {
                    self.get_token();
                    UseIdentifier::BuiltInSuper
                }
                Lexem::KwSelf => {
                    self.get_token();
                    UseIdentifier::BuiltInSelf
                }
                Lexem::KwCrate => {
                    self.get_token();
                    UseIdentifier::BuiltInCrate
                }
                Lexem::Star => {
                    self.get_token();

                    let next = self.peek_token();
                    let req = [Lexem::EndOfLine, Lexem::EndOfFile];
                    if !req.contains(&next.lexem) {
                        return Err(Message::unexpected_token(next, &req));
                    }

                    self.get_token();

                    UseIdentifier::BuiltInAll
                }
                Lexem::Identifier(id) => {
                    self.get_token();
                    UseIdentifier::Identifier(Ident::from(id))
                }
                _ => {
                    return Err(Message::unexpected_token(
                        next,
                        &[
                            Lexem::KwSuper,
                            Lexem::KwSelf,
                            Lexem::KwCrate,
                            Lexem::Identifier("".into()),
                        ],
                    ))
                }
            };

            u.identifier.push(id);

            let next = self.peek_token();
            match next.lexem {
                Lexem::EndOfLine | Lexem::EndOfFile => {
                    self.get_token();
                    break;
                }
                Lexem::Dcolon => {
                    self.get_token();
                }
                _ => {
                    return Err(Message::unexpected_token(
                        next,
                        &[Lexem::Dcolon, Lexem::EndOfLine],
                    ))
                }
            }
        }

        Ok(())
    }
}
