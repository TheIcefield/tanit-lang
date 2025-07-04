use tanitc_ast::{Ast, ExternDef};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    fn parse_extern_header(&mut self, extern_def: &mut ExternDef) -> Result<(), Message> {
        extern_def.location = self.consume_token(Lexem::KwExtern)?.location;
        extern_def.abi_name = self.consume_text()?;

        Ok(())
    }

    fn parse_extern_body_internal(&mut self, extern_def: &mut ExternDef) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            if matches!(next.lexem, Lexem::Rcb | Lexem::EndOfFile) {
                break;
            }

            if next.lexem == Lexem::EndOfLine {
                self.get_token();
                continue;
            }

            let attrs = self.parse_attributes()?;

            let next = self.peek_token();

            let statement = match next.lexem {
                Lexem::KwFunc => self.parse_func_def(),

                _ => {
                    self.skip_until(&[Lexem::EndOfLine]);
                    self.get_token();

                    self.error(Message::unexpected_token(next, &[Lexem::KwFunc]));
                    continue;
                }
            };

            match statement {
                Ok(mut child) => {
                    child.apply_attributes(attrs)?;

                    let Ast::FuncDef(child) = child else {
                        unreachable!();
                    };

                    extern_def.functions.push(child);
                }
                Err(err) => self.error(err),
            }
        }

        Ok(())
    }

    fn parse_extern_body(&mut self, extern_def: &mut ExternDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        self.parse_extern_body_internal(extern_def)?;

        self.consume_token(Lexem::Rcb)?;
        Ok(())
    }

    pub fn parse_extern_def(&mut self) -> Result<Ast, Message> {
        let mut node = ExternDef::default();

        self.parse_extern_header(&mut node)?;
        self.parse_extern_body(&mut node)?;

        Ok(Ast::from(node))
    }
}
