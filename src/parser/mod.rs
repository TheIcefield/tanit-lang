use crate::ast::{scopes, Ast};
use crate::messages::{Errors, Message, Warnings};

pub mod lexer;
pub mod location;
pub mod token;

use lexer::Lexer;
use location::Location;
use token::{Lexem, Token};

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

    pub fn get_path(&self) -> String {
        self.lexer.get_path()
    }

    pub fn is_token_verbose(&self) -> bool {
        self.lexer.verbose
    }

    pub fn does_ignore_nl(&self) -> bool {
        self.lexer.ignores_nl
    }

    pub fn set_ignore_nl_option(&mut self, opt: bool) {
        self.lexer.ignores_nl = opt;
    }

    pub fn parse(&mut self) -> Result<Ast, (Errors, Warnings)> {
        let ast = {
            match scopes::Scope::parse_global_internal(self) {
                Ok(statements) => Ok(Ast::Scope {
                    node: scopes::Scope {
                        statements,
                        is_global: true,
                    },
                }),
                Err(err) => Err((vec![err], Warnings::new())),
            }
        };

        if ast.is_err() || !self.errors.is_empty() {
            return Err((
                std::mem::take(&mut self.errors),
                std::mem::take(&mut self.warnings),
            ));
        }

        Ok(ast.unwrap())
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

    pub fn consume_identifier(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Identifier(_) => Ok(self.get_token()),

            _ => Err(Message::new(
                tkn.location,
                &format!("Unexpected token {}. Expected identifier.", tkn),
            )),
        }
    }

    pub fn consume_integer(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Integer(_) => Ok(self.get_token()),

            _ => Err(Message::new(
                tkn.location,
                &format!("Unexpected token {}. Expected integer number.", tkn),
            )),
        }
    }

    pub fn consume_decimal(&mut self) -> Result<Token, Message> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Decimal(_) => Ok(self.get_token()),

            _ => Err(Message::new(
                tkn.location,
                &format!("Unexpected token {}. Expected decimal number.", tkn),
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
}
