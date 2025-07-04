use tanitc_ident::Ident;
use tanitc_messages::{Errors, Message, Warnings};

pub mod ast;

use tanitc_lexer::{
    location::Location,
    token::{Lexem, Token},
    Lexer,
};

pub struct Parser {
    lexer: Lexer,
    errors: Errors,
    warnings: Warnings,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            errors: Errors::new(),
            warnings: Warnings::new(),
        }
    }

    pub fn from_text(src: &'static str) -> Result<Self, &'static str> {
        Ok(Self {
            lexer: Lexer::from_text(src)?,
            errors: Errors::new(),
            warnings: Warnings::new(),
        })
    }

    pub fn get_path(&self) -> String {
        self.lexer.get_path()
    }

    pub fn does_ignore_nl(&self) -> bool {
        self.lexer.ignores_nl
    }

    pub fn set_ignore_nl_option(&mut self, opt: bool) {
        self.lexer.ignores_nl = opt;
    }

    pub fn get_location(&self) -> Location {
        self.lexer.get_location()
    }

    pub fn get_token(&mut self) -> Token {
        let next = self.lexer.peek();

        if next.lexem == Lexem::EndOfFile {
            return next;
        }

        self.lexer.get()
    }

    pub fn get_singular(&mut self) -> Token {
        let next = self.lexer.peek_singular();

        if next.lexem == Lexem::EndOfFile {
            return next;
        }

        self.lexer.get_singular()
    }

    pub fn peek_token(&mut self) -> Token {
        self.lexer.peek()
    }

    pub fn peek_singular(&mut self) -> Token {
        self.lexer.peek_singular()
    }

    pub fn consume_new_line(&mut self) -> Result<Token, Message> {
        let old_opt = self.lexer.ignores_nl;
        self.lexer.ignores_nl = false;
        let nl = self.consume_singular(Lexem::EndOfLine);
        self.lexer.ignores_nl = old_opt;
        nl
    }

    pub fn consume_singular(&mut self, token_type: Lexem) -> Result<Token, Message> {
        let tkn = self.peek_singular();

        if tkn.lexem == token_type {
            let tkn = self.get_singular();

            return Ok(tkn);
        }

        Err(Message::unexpected_token(tkn, &[token_type]))
    }

    pub fn consume_token(&mut self, token_type: Lexem) -> Result<Token, Message> {
        let tkn = self.lexer.peek();

        if tkn.lexem == token_type {
            let tkn = self.lexer.get();

            return Ok(tkn);
        }

        Err(Message::unexpected_token(tkn, &[token_type]))
    }

    pub fn consume_identifier(&mut self) -> Result<Ident, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Identifier(id) => {
                self.get_token();
                Ok(Ident::from(id))
            }

            _ => Err(Message::from_string(
                tkn.location,
                format!("Unexpected token {tkn}. Expected identifier."),
            )),
        }
    }

    pub fn consume_integer(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Integer(_) => Ok(self.get_token()),

            _ => Err(Message::from_string(
                tkn.location,
                format!("Unexpected token {tkn}. Expected integer number."),
            )),
        }
    }

    pub fn consume_decimal(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Decimal(_) => Ok(self.get_token()),

            _ => Err(Message::from_string(
                tkn.location,
                format!("Unexpected token {tkn}. Expected decimal number."),
            )),
        }
    }

    pub fn consume_text(&mut self) -> Result<String, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Text(text) => {
                self.get_token();
                Ok(text)
            }

            _ => Err(Message::from_string(
                tkn.location,
                format!("Unexpected token {tkn}. Expected text."),
            )),
        }
    }

    pub fn skip_until(&mut self, until: &[Lexem]) {
        let old_opt = self.lexer.ignores_nl;

        if until.contains(&Lexem::EndOfLine) {
            self.lexer.ignores_nl = false;
        }

        loop {
            let token = self.peek_token();

            if until.contains(&token.lexem) || until.contains(&Lexem::EndOfFile) {
                self.lexer.ignores_nl = old_opt;
                return;
            }

            self.get_token();
        }
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
