use std::path::{Path, PathBuf};

use tanitc_ast::program_ctx::ProgramCtx;
use tanitc_messages::{listener::MessageListener, Message};

pub(crate) mod program_ctx;

use tanitc_lexer::{
    token::{lexeme::Lexeme, Token},
    Lexer, Tokens,
};

pub struct Parser {
    path: PathBuf,
    tokens: Tokens,
    offset: usize,
    messages: MessageListener,
    ignore_nl_opt: bool,
}

pub type ParseResult<T> = Result<T, Message>;

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        Self {
            path: lexer.get_path().to_path_buf(),
            tokens: lexer.tokenize(),
            offset: 0,
            messages: MessageListener::new(),
            ignore_nl_opt: true,
        }
    }

    pub fn from_text(src: &str) -> Self {
        Self {
            path: PathBuf::from("text"),
            tokens: Lexer::new(src.chars().peekable(), &PathBuf::from("text")).tokenize(),
            offset: 0,
            messages: MessageListener::new(),
            ignore_nl_opt: true,
        }
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn parse_program(&mut self) -> Result<Box<ProgramCtx>, MessageListener> {
        match self.parse_program_ctx() {
            Ok(program_ctx) => Ok(Box::new(program_ctx)),
            Err(msg) => {
                self.error(msg);
                Err(std::mem::take(self.messages_mut()))
            }
        }
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

            if self.ignore_nl_opt && Lexeme::EndOfLine == *tkn.lexeme_ref() {
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

            if self.ignore_nl_opt && Lexeme::EndOfLine == *tkn.lexeme_ref() {
                self.offset += 1;
                continue;
            }

            return Some(tkn.clone());
        }
    }

    pub(crate) fn is_eof(&mut self) -> bool {
        self.peek_token().is_none()
    }

    pub(crate) fn is_next(&mut self, token_type: Lexeme) -> bool {
        let Some(tkn) = self.peek_token() else {
            return false;
        };

        *tkn.lexeme_ref() == token_type
    }

    pub(crate) fn consume_token(&mut self, token_type: Lexeme) -> Result<Token, Message> {
        if self.is_eof() {
            return Err(Message::reached_eof());
        }

        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        if *tkn.lexeme_ref() == token_type {
            self.get_token();
            return Ok(tkn);
        }

        Err(Message::unexpected_token(&tkn, &[token_type]))
    }

    pub(crate) fn consume_identifier(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexeme_ref() {
            Lexeme::Identifier(_) => {
                self.get_token();
                Ok(tkn)
            }

            _ => Err(Message::new(
                tkn.get_location(),
                format!("Unexpected token {tkn}. Expected identifier."),
            )),
        }
    }

    pub(crate) fn consume_integer(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexeme_ref() {
            Lexeme::Integer(_) => {
                self.get_token();
                Ok(tkn)
            }
            _ => Err(Message::new(
                tkn.get_location(),
                format!("Unexpected token {tkn}. Expected integer number."),
            )),
        }
    }

    pub(crate) fn consume_decimal(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexeme_ref() {
            Lexeme::Decimal(_) => {
                self.get_token();
                Ok(tkn)
            }
            _ => Err(Message::new(
                tkn.get_location(),
                format!("Unexpected token {tkn}. Expected decimal number."),
            )),
        }
    }

    pub(crate) fn consume_text(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token().ok_or(Message::reached_eof())?;

        match tkn.lexeme_ref() {
            Lexeme::Text(_) => {
                self.get_token();
                Ok(tkn)
            }
            _ => Err(Message::new(
                tkn.get_location(),
                format!("Unexpected token {tkn}. Expected text."),
            )),
        }
    }

    pub(crate) fn consume_new_line(&mut self) -> Result<Token, Message> {
        let old_opt = self.ignore_nl_opt;
        self.ignore_nl_opt = false;
        let nl = self.consume_token(Lexeme::EndOfLine);
        self.ignore_nl_opt = old_opt;
        nl
    }

    pub(crate) fn skip_until(&mut self, until: &[Lexeme]) {
        let old_opt = self.ignore_nl_opt;

        if until.contains(&Lexeme::EndOfLine) {
            self.ignore_nl_opt = false;
        }

        loop {
            let Some(token) = self.peek_token() else {
                break;
            };

            if until.contains(token.lexeme_ref()) {
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

    pub fn messages_ref(&self) -> &MessageListener {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut MessageListener {
        &mut self.messages
    }

    pub fn error(&mut self, mut error: Message) {
        error.text = format!("Syntax error: {}", error.text);
        self.messages.error(error);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Syntax warning: {}", warn.text);
        self.messages.warn(warn);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tanitc_lexer::{token::lexeme::Lexeme, Lexer};

    use crate::Parser;

    #[test]
    fn set_offset_test() {
        const SRC: &str = "hello, world!";

        let lexer = Lexer::new(SRC.chars().peekable(), &PathBuf::from("test"));

        let mut parser = Parser::new(lexer);

        assert!(parser.get_token().unwrap().is_identifier());

        let index = parser.get_current_token_index();

        assert_eq!(*parser.get_token().unwrap().lexeme_ref(), Lexeme::Comma);
        assert!(parser.get_token().unwrap().is_identifier());

        parser.set_current_token_index(index);

        assert_eq!(*parser.get_token().unwrap().lexeme_ref(), Lexeme::Comma);
        assert!(parser.get_token().unwrap().is_identifier());
    }
}
