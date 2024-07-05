use crate::ast::{scopes, Ast};
use crate::error_listener::{
    ErrorListener, CANNOT_CONVERT_TO_DECIMAL_ERROR_STR, CANNOT_CONVERT_TO_INTEGER_ERROR_STR,
    PARSING_FAILED_ERROR_STR, UNEXPECTED_TOKEN_ERROR_STR,
};
use crate::lexer::{self, Lexem, Lexer, Location, Token};

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

    pub fn get_path(&self) -> Result<String, &'static str> {
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

    pub fn parse(&mut self) -> Result<Ast, ErrorListener> {
        let ast = {
            if let Ok(statements) = scopes::Scope::parse_global_internal(self) {
                Ok(Ast::Scope {
                    node: scopes::Scope {
                        statements,
                        is_global: true,
                    },
                })
            } else {
                Err(PARSING_FAILED_ERROR_STR)
            }
        };

        if ast.is_err() || !self.error_listener.is_empty() {
            return Err(std::mem::take(&mut self.error_listener));
        }

        Ok(ast.unwrap())
    }

    pub fn error_listener(&mut self) -> ErrorListener {
        std::mem::take(&mut self.error_listener)
    }

    pub fn get_location(&self) -> lexer::Location {
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

    pub fn consume_new_line(&mut self) -> Result<Token, &'static str> {
        let old_opt = self.lexer.ignores_nl;
        self.lexer.ignores_nl = false;
        let nl = self.consume_singular(Lexem::EndOfLine);
        self.lexer.ignores_nl = old_opt;
        nl
    }

    pub fn consume_singular(&mut self, token_type: Lexem) -> Result<Token, &'static str> {
        let tkn = self.peek_singular();

        if tkn.lexem == token_type {
            let tkn = self.get_singular();

            return Ok(tkn);
        }

        self.error_listener.syntax_error(
            &format!(
                "Unexpected token: \"{}\", but was expected: \"{}\"",
                tkn.lexem, token_type
            ),
            tkn.location,
        );

        Err(UNEXPECTED_TOKEN_ERROR_STR)
    }

    pub fn consume_token(&mut self, token_type: Lexem) -> Result<Token, &'static str> {
        let tkn = self.lexer.peek();

        if tkn.lexem == token_type {
            let tkn = self.lexer.get();

            return Ok(tkn);
        }

        self.error_listener.syntax_error(
            &format!(
                "Unexpected token: \"{}\", but was expected: \"{}\"",
                tkn.lexem, token_type
            ),
            tkn.location,
        );

        Err(UNEXPECTED_TOKEN_ERROR_STR)
    }

    pub fn consume_identifier(&mut self) -> Result<Lexem, &'static str> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Identifier(_) => Ok(self.get_token().lexem),

            _ => {
                self.error_listener.syntax_error(
                    &format!(
                        "Unexpected token: \"{}\", but was expected: \"identifier\"",
                        tkn.lexem
                    ),
                    tkn.location,
                );

                Err(UNEXPECTED_TOKEN_ERROR_STR)
            }
        }
    }

    pub fn consume_integer(&mut self) -> Result<usize, &'static str> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Integer(val) => {
                self.get_token();
                let val = val.parse::<usize>();
                if val.is_err() {
                    return Err(CANNOT_CONVERT_TO_INTEGER_ERROR_STR);
                }
                Ok(val.unwrap())
            }

            _ => {
                self.error_listener.syntax_error(
                    &format!(
                        "Unexpected token: \"{}\", but was expected: \"identifier\"",
                        tkn.lexem
                    ),
                    tkn.location,
                );

                Err(UNEXPECTED_TOKEN_ERROR_STR)
            }
        }
    }

    pub fn consume_decimal(&mut self) -> Result<f64, &'static str> {
        let tkn = self.peek_token();

        match tkn.lexem {
            Lexem::Decimal(val) => {
                self.get_token();
                let val = val.parse::<f64>();
                if val.is_err() {
                    return Err(CANNOT_CONVERT_TO_DECIMAL_ERROR_STR);
                }
                Ok(val.unwrap())
            }

            _ => {
                self.error_listener.syntax_error(
                    &format!(
                        "Unexpected token: \"{}\", but was expected: \"identifier\"",
                        tkn.lexem
                    ),
                    tkn.location,
                );

                Err(UNEXPECTED_TOKEN_ERROR_STR)
            }
        }
    }

    pub fn skip_until(&mut self, until: Lexem) {
        let old_opt = self.lexer.ignores_nl;

        if Lexem::EndOfLine == until {
            self.lexer.ignores_nl = false;
        }

        loop {
            let token = self.peek_token().lexem;

            if until == token || Lexem::EndOfFile == token {
                self.lexer.ignores_nl = old_opt;
                return;
            }

            self.get_token();
        }
    }

    pub fn error(&mut self, message: &str, location: Location) {
        self.error_listener.syntax_error(message, location);
    }

    pub fn push_error(&mut self, error: String) {
        self.error_listener.push_error(error)
    }
}

pub fn put_intent(intent: usize) -> String {
    let mut res = "".to_string();
    for _ in 0..intent {
        res.push(' ');
    }
    res
}

pub fn dump_ast(output: String, ast: &Ast) -> std::io::Result<()> {
    let mut stream = std::fs::File::create(format!("{}_ast.xml", output)).unwrap();
    ast.traverse(&mut stream, 0)
}
