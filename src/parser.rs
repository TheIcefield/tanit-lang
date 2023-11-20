use crate::ast::{scopes, Ast};
use crate::error_listener::ErrorListener;
use crate::lexer::{Lexer, Location, Token, TokenType};

type ParseResult = Result<Ast, ErrorListener>;

pub type Id = String;

pub struct Parser {
    error_listener: ErrorListener,
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer, error_listener: ErrorListener) -> Self {
        Self {
            error_listener,
            lexer,
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        let ast = parse_program(self);

        if ast.is_none() || !self.error_listener.is_empty() {
            return Err(std::mem::take(&mut self.error_listener));
        }

        Ok(ast.unwrap())
    }

    pub fn get_token(&mut self) -> Token {
        let next = self.lexer.peek();

        if next.lexem == TokenType::EndOfFile {
            return next;
        }

        self.lexer.get()
    }

    pub fn get_singular(&mut self) -> Token {
        let next = self.lexer.peek_singular();

        if next.lexem == TokenType::EndOfFile {
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

    pub fn consume_new_line(&mut self) -> Option<Token> {
        self.consume_singular(TokenType::EndOfLine)
    }

    pub fn consume_singular(&mut self, token_type: TokenType) -> Option<Token> {
        let tkn = self.peek_singular();

        if tkn.lexem == token_type {
            let tkn = self.get_singular();

            return Some(tkn);
        }

        self.error_listener.syntax_error(
            &format!(
                "Unexpected token: \"{}\", but was expected: \"{}\"",
                tkn, token_type
            ),
            tkn.location,
        );

        None
    }

    pub fn consume_token(&mut self, token_type: TokenType) -> Option<Token> {
        loop {
            let tkn = self.lexer.peek();

            if tkn.lexem == token_type {
                return Some(self.lexer.get());
            } else if tkn.lexem == TokenType::EndOfLine {
                self.lexer.get();
            } else {
                break;
            }
        }

        let tkn = self.lexer.peek();

        if tkn.lexem == token_type {
            let tkn = self.lexer.get();

            return Some(tkn);
        }

        self.error_listener.syntax_error(
            &format!(
                "Unexpected token: \"{}\", but was expected: \"{}\"",
                tkn, token_type
            ),
            tkn.location,
        );

        None
    }

    pub fn consume_identifier(&mut self) -> Option<Id> {
        loop {
            let tkn = self.lexer.peek();

            if tkn.lexem == TokenType::EndOfLine {
                self.lexer.get();
            } else {
                break;
            }
        }

        let tkn = self.peek_token();

        match tkn.lexem {
            TokenType::Identifier(id) => {
                self.get_token();
                Some(id)
            }

            _ => {
                self.error_listener.syntax_error(
                    &format!(
                        "Unexpected token: \"{}\", but was expected: \"identifier\"",
                        tkn
                    ),
                    tkn.location,
                );

                None
            }
        }
    }

    pub fn error(&mut self, message: &str, location: Location) {
        self.error_listener.syntax_error(message, location);
    }
}

pub fn put_intent(intent: usize) -> String {
    let mut res = "".to_string();
    for _ in 0..intent - 1 {
        res.push(' ');
    }
    res
}

pub fn dump_ast(output: String, ast: &Ast) -> std::io::Result<()> {
    let mut stream = std::fs::File::create(format!("{}.xml", output)).unwrap();
    ast.traverse(&mut stream, 0)
}

fn parse_program(parser: &mut Parser) -> Option<Ast> {
    Some(Ast::GScope {
        node: scopes::Scope {
            statements: scopes::parse_global_internal(parser)?,
        },
    })
}
