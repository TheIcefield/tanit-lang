use std::path::{Path, PathBuf};

use tanitc_ident::Ident;
use tanitc_messages::{Errors, Message, Warnings};

pub mod ast;

use tanitc_lexer::{
    token::{Lexem, Token},
    Lexer, Tokens,
};

pub struct Parser {
    path: PathBuf,
    tokens: Tokens,
    offset: usize,
    errors: Errors,
    warnings: Warnings,
    ignore_nl_opt: bool,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        Self {
            path: lexer.get_path().to_path_buf(),
            tokens: lexer.tokenize(),
            offset: 0,
            errors: Errors::new(),
            warnings: Warnings::new(),
            ignore_nl_opt: true,
        }
    }

    pub fn from_text(src: &str) -> Self {
        Self {
            path: PathBuf::from("text"),
            tokens: Lexer::new(src.chars().peekable(), &PathBuf::from("text")).tokenize(),
            offset: 0,
            errors: Errors::new(),
            warnings: Warnings::new(),
            ignore_nl_opt: true,
        }
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn does_ignore_nl(&self) -> bool {
        self.ignore_nl_opt
    }

    pub(crate) fn set_ignore_nl_option(&mut self, opt: bool) {
        self.ignore_nl_opt = opt;
    }

    pub(crate) fn get_token(&mut self) -> Option<Token> {
        loop {
            let tkn = self.tokens.get(self.offset)?;

            if self.ignore_nl_opt && Lexem::EndOfLine == *tkn.lexem_ref() {
                self.offset += 1;
                continue;
            }

            self.offset += 1;
            return Some(tkn.clone());
        }
    }

    pub(crate) fn peek_token(&mut self) -> Option<Token> {
        loop {
            let tkn = self.tokens.get(self.offset)?;

            if self.ignore_nl_opt && Lexem::EndOfLine == *tkn.lexem_ref() {
                self.offset += 1;
                continue;
            }

            return Some(tkn.clone());
        }
    }

    pub(crate) fn is_eof(&mut self) -> bool {
        self.peek_token().is_none()
    }

    pub(crate) fn is_next(&mut self, token_type: Lexem) -> bool {
        let Some(tkn) = self.peek_token() else {
            return false;
        };

        *tkn.lexem_ref() == token_type
    }

    pub(crate) fn consume_token(&mut self, token_type: Lexem) -> Result<Token, Message> {
        if self.is_eof() {
            return Err(Message::reached_eof());
        }

        let tkn = self.peek_token().unwrap();

        if *tkn.lexem_ref() == token_type {
            self.get_token();
            return Ok(tkn);
        }

        Err(Message::unexpected_token(&tkn, &[token_type]))
    }

    pub(crate) fn consume_identifier(&mut self) -> Result<Ident, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexem_ref() {
            Lexem::Identifier(id) => {
                self.get_token();
                Ok(Ident::from(id.clone()))
            }

            _ => Err(Message::from_string(
                tkn.location_ref(),
                format!("Unexpected token {tkn}. Expected identifier."),
            )),
        }
    }

    pub(crate) fn consume_integer(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexem_ref() {
            Lexem::Integer(_) => {
                self.get_token();
                Ok(tkn)
            }
            _ => Err(Message::from_string(
                tkn.location_ref(),
                format!("Unexpected token {tkn}. Expected integer number."),
            )),
        }
    }

    pub(crate) fn consume_decimal(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexem_ref() {
            Lexem::Decimal(_) => {
                self.get_token();
                Ok(tkn)
            }
            _ => Err(Message::from_string(
                tkn.location_ref(),
                format!("Unexpected token {tkn}. Expected decimal number."),
            )),
        }
    }

    pub(crate) fn consume_text(&mut self) -> Result<String, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexem_ref() {
            Lexem::Text(text) => {
                self.get_token();
                Ok(text.clone())
            }
            _ => Err(Message::from_string(
                tkn.location_ref(),
                format!("Unexpected token {tkn}. Expected text."),
            )),
        }
    }

    pub(crate) fn consume_new_line(&mut self) -> Result<Token, Message> {
        let old_opt = self.ignore_nl_opt;
        self.ignore_nl_opt = false;
        let nl = self.consume_token(Lexem::EndOfLine);
        self.ignore_nl_opt = old_opt;
        nl
    }

    pub(crate) fn skip_until(&mut self, until: &[Lexem]) {
        let old_opt = self.ignore_nl_opt;

        if until.contains(&Lexem::EndOfLine) {
            self.ignore_nl_opt = false;
        }

        loop {
            let token = self.peek_token();
            if token.is_some() {
                break;
            }

            let token = token.unwrap();
            if until.contains(token.lexem_ref()) {
                break;
            }

            self.get_token();
        }

        self.ignore_nl_opt = old_opt;
    }

    pub fn get_current_token_index(&self) -> usize {
        self.offset
    }

    pub fn set_current_token_index(&mut self, index: usize) {
        self.offset = index;
    }

    pub fn error(&mut self, mut err: Message) {
        err.text = format!("Syntax error: {}", err.text);
        self.errors.push(err);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Syntax warning: {}", warn.text);
        self.errors.push(warn);
    }

    pub fn get_errors(&mut self) -> Errors {
        std::mem::take(&mut self.errors)
    }

    pub fn get_warnings(&mut self) -> Warnings {
        std::mem::take(&mut self.warnings)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tanitc_lexer::{token::Lexem, Lexer};

    use crate::Parser;

    #[test]
    fn set_offset_test() {
        const SRC: &str = "hello, world!";

        let lexer = Lexer::new(SRC.chars().peekable(), &PathBuf::from("test"));

        let mut parser = Parser::new(lexer);

        assert!(parser.get_token().unwrap().is_identifier());

        let index = parser.get_current_token_index();

        assert_eq!(*parser.get_token().unwrap().lexem_ref(), Lexem::Comma);
        assert!(parser.get_token().unwrap().is_identifier());

        parser.set_current_token_index(index);

        assert_eq!(*parser.get_token().unwrap().lexem_ref(), Lexem::Comma);
        assert!(parser.get_token().unwrap().is_identifier());
    }
}
