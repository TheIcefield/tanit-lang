pub mod location;
pub mod token;

use std::{iter::Peekable, path::Path, str::Chars};

use location::Location;
use token::{Lexem, Token};

pub type Tokens = Vec<Token>;

pub struct Lexer<'a> {
    location: Location,
    next_token: Option<Token>,
    input: Peekable<Chars<'a>>,
    pub verbose_tokens: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: Peekable<Chars<'a>>, path: &Path) -> Self {
        Self {
            location: Location::new(path),
            next_token: None,
            verbose_tokens: false,
            input,
        }
    }

    pub fn tokenize(&mut self) -> Tokens {
        let mut tokens = Tokens::new();

        loop {
            let Some(token) = self.get() else {
                break;
            };

            tokens.push(token);
        }

        tokens
    }

    pub fn get(&mut self) -> Option<Token> {
        let tkn = self.get_next(false);

        if self.verbose_tokens {
            if let Some(tkn) = &tkn {
                println!("{tkn}");
            } else {
                println!("EOF");
            };
        }

        tkn
    }

    pub fn get_singular(&mut self) -> Option<Token> {
        let tkn = self.get_next(true);

        if self.verbose_tokens {
            if let Some(tkn) = &tkn {
                println!("{tkn}");
            } else {
                println!("EOF");
            };
        }

        tkn
    }

    pub fn peek(&mut self) -> Option<Token> {
        if self.next_token.is_some() {
            let tkn = self.next_token.clone().unwrap();
            return Some(tkn);
        }

        self.next_token = self.get_next(false);
        self.next_token.clone()
    }

    pub fn peek_singular(&mut self) -> Option<Token> {
        if self.next_token.is_some() {
            let tkn = self.next_token.clone().unwrap();
            return Some(tkn);
        }

        self.next_token = self.get_next(true);
        self.next_token.clone()
    }

    pub fn location_ref(&self) -> &Location {
        &self.location
    }

    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.location
    }

    pub fn get_path(&self) -> &Path {
        if self.location.path.as_os_str().is_empty() {
            panic!("Lexer input stream is not a file");
        }

        self.location.path.as_path()
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn next_char(&mut self) -> Option<char> {
        let next = self.input.next();

        if next.is_some_and(|ch| ch == '\n') {
            self.location.new_line();
        } else if next.is_some() {
            self.location.shift();
        }

        next
    }

    fn skip_spaces(&mut self) {
        while self.peek_char().is_some_and(|ch| ch.is_ascii_whitespace()) {
            if self.peek_char().is_some_and(|ch| *ch == '\n') {
                self.location.new_line();
                return;
            }

            self.next_char();
        }
    }

    fn skip_comment(&mut self) {
        let Some(ch) = self.peek_char() else {
            return;
        };

        if *ch == '#' {
            while self.next_char() != Some('\n') {}

            self.location.new_line();
        }
    }

    fn get_next(&mut self, singular: bool) -> Option<Token> {
        if self.next_token.is_some() {
            let token = self.next_token.clone().unwrap();
            self.next_token = None;
            return Some(token);
        }

        loop {
            self.skip_spaces();
            self.skip_comment();

            let ch = self.peek_char();

            if ch.is_some_and(|ch| *ch != '#' && !ch.is_ascii_whitespace() || *ch == '\n')
                || ch.is_none()
            {
                break;
            }
        }

        let next_char = self.peek_char()?;

        let lexem = match next_char {
            '\n' => {
                self.next_char();
                Lexem::EndOfLine
            }
            '(' => {
                self.next_char();
                Lexem::LParen
            }
            ')' => {
                self.next_char();
                Lexem::RParen
            }
            '[' => {
                self.next_char();
                Lexem::Lsb
            }
            ']' => {
                self.next_char();
                Lexem::Rsb
            }
            '{' => {
                self.next_char();
                Lexem::Lcb
            }
            '}' => {
                self.next_char();
                Lexem::Rcb
            }
            '.' => {
                self.next_char();
                Lexem::Dot
            }
            ',' => {
                self.next_char();
                Lexem::Comma
            }
            '>' => {
                self.next_char();
                let mut lexem = Lexem::Gt;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::Gte;
                } else if self.peek_char().is_some_and(|ch| *ch == '>') && !singular {
                    self.next_char();
                    lexem = Lexem::RShift;

                    if self.peek_char().is_some_and(|ch| *ch == '=') {
                        self.next_char();
                        lexem = Lexem::RShiftAssign;
                    }
                }

                lexem
            }
            '<' => {
                self.next_char();
                let mut lexem = Lexem::Lt;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::Lte;
                } else if self.peek_char().is_some_and(|ch| *ch == '<') && !singular {
                    self.next_char();
                    lexem = Lexem::LShift;

                    if self.peek_char().is_some_and(|ch| *ch == '=') {
                        self.next_char();
                        lexem = Lexem::LShiftAssign;
                    }
                }

                lexem
            }
            '+' => {
                self.next_char();
                let mut lexem = Lexem::Plus;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::AddAssign;
                }

                lexem
            }
            '-' => {
                self.next_char();
                let mut lexem = Lexem::Minus;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::SubAssign;
                }

                lexem
            }
            '/' => {
                self.next_char();
                let mut lexem = Lexem::Slash;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::DivAssign;
                }

                lexem
            }
            '%' => {
                self.next_char();
                let mut lexem = Lexem::Percent;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::ModAssign;
                }

                lexem
            }
            '*' => {
                self.next_char();
                let mut lexem = Lexem::Star;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::MulAssign;
                }

                lexem
            }
            '!' => {
                self.next_char();
                let mut lexem = Lexem::Not;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::Neq;
                }

                lexem
            }
            '=' => {
                self.next_char();
                let mut lexem = Lexem::Assign;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::Eq;
                }

                lexem
            }
            '&' => {
                self.next_char();
                let mut lexem = Lexem::Ampersand;

                if self.peek_char().is_some_and(|ch| *ch == '&') && !singular {
                    self.next_char();
                    lexem = Lexem::And;
                } else if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::AndAssign;
                }

                lexem
            }
            '^' => {
                self.next_char();
                let mut lexem = Lexem::Xor;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::XorAssign;
                }

                lexem
            }
            '|' => {
                self.next_char();
                let mut lexem = Lexem::Stick;

                if self.peek_char().is_some_and(|ch| *ch == '=') && !singular {
                    self.next_char();
                    lexem = Lexem::OrAssign;
                } else if self.peek_char().is_some_and(|ch| *ch == '|') && !singular {
                    self.next_char();
                    lexem = Lexem::Or;
                }

                lexem
            }
            ':' => {
                self.next_char();
                let mut lexem = Lexem::Colon;

                if self.peek_char().is_some_and(|ch| *ch == ':') && !singular {
                    self.next_char();
                    lexem = Lexem::Dcolon;
                }

                lexem
            }

            _ if '\"' == *next_char => self.get_text_lexem()?,
            _ if next_char.is_ascii_digit() => self.get_numeric_lexem()?,
            _ if next_char.is_ascii_alphabetic() || '_' == *next_char => self.get_string_lexem()?,
            _ => Lexem::Unknown,
        };

        Some(Token::new(lexem, self.location_ref().clone()))
    }

    fn get_numeric_lexem(&mut self) -> Option<Lexem> {
        let mut text = String::new();
        let mut is_float = false;

        while self
            .peek_char()
            .is_some_and(|ch| ch.is_ascii_digit() || *ch == '.')
        {
            if self.peek_char().is_some_and(|ch| *ch == '.') {
                if is_float {
                    return None;
                }
                is_float = true;
            }

            text.push(self.next_char()?);
        }

        Some(if is_float {
            Lexem::Decimal(text)
        } else {
            Lexem::Integer(text)
        })
    }

    fn get_string_lexem(&mut self) -> Option<Lexem> {
        let mut text = String::new();

        while self
            .peek_char()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        {
            text.push(self.next_char()?);
        }

        Some(match &text[..] {
            "def" => Lexem::KwDef,
            "module" => Lexem::KwModule,
            "struct" => Lexem::KwStruct,
            "union" => Lexem::KwUnion,
            "variant" => Lexem::KwVariant,
            "impl" => Lexem::KwImpl,
            "enum" => Lexem::KwEnum,
            "var" => Lexem::KwVar,
            "mut" => Lexem::KwMut,
            "const" => Lexem::KwConst,
            "alias" => Lexem::KwAlias,
            "func" => Lexem::KwFunc,
            "if" => Lexem::KwIf,
            "else" => Lexem::KwElse,
            "loop" => Lexem::KwLoop,
            "do" => Lexem::KwDo,
            "while" => Lexem::KwWhile,
            "for" => Lexem::KwFor,
            "continue" => Lexem::KwContinue,
            "break" => Lexem::KwBreak,
            "return" => Lexem::KwReturn,
            "extern" => Lexem::KwExtern,
            "static" => Lexem::KwStatic,
            "use" => Lexem::KwUse,
            "super" => Lexem::KwSuper,
            "Self" => Lexem::KwSelfT,
            "self" => Lexem::KwSelfO,
            "crate" => Lexem::KwCrate,
            "as" => Lexem::KwAs,
            "safe" => Lexem::KwSafe,
            "unsafe" => Lexem::KwUnsafe,
            "pub" => Lexem::KwPub,
            _ => Lexem::Identifier(text),
        })
    }

    fn get_text_lexem(&mut self) -> Option<Lexem> {
        let mut text = String::new();

        // push opening '\"'
        text.push(self.next_char()?);

        while self.peek_char().is_some_and(|ch| ch.is_ascii()) {
            if self.peek_char().is_some_and(|ch| *ch == '\"') {
                // push closing '\"'
                text.push(self.next_char()?);
                break;
            }

            // push char
            text.push(self.next_char()?);
        }

        Some(Lexem::Text(text))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{location::Location, token::Lexem, Lexer};

    #[test]
    fn lexer_test() {
        const SRC_TEXT: &str = "+ hello func var 65 -= <<\n struct alpha safe unsafe impl";

        let test_path = PathBuf::from("test");

        let mut lexer = Lexer::new(SRC_TEXT.chars().peekable(), &test_path);

        let mut tkn = lexer.get().unwrap();
        let mut location = Location::new(&test_path);

        location.col = 1;
        assert_eq!(*tkn.lexem_ref(), Lexem::Plus);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 7;
        assert_eq!(*tkn.lexem_ref(), Lexem::identifier("hello"));
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 12;
        assert_eq!(*tkn.lexem_ref(), Lexem::KwFunc);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 16;
        assert_eq!(*tkn.lexem_ref(), Lexem::KwVar);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 19;
        assert_eq!(*tkn.lexem_ref(), Lexem::integer(65));
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 22;
        assert_eq!(*tkn.lexem_ref(), Lexem::SubAssign);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 25;
        assert_eq!(*tkn.lexem_ref(), Lexem::LShift);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.row = 2;
        location.col = 0;
        assert_eq!(*tkn.lexem_ref(), Lexem::EndOfLine);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 7;
        assert_eq!(*tkn.lexem_ref(), Lexem::KwStruct);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 13;
        assert_eq!(*tkn.lexem_ref(), Lexem::identifier("alpha"));
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 18;
        assert_eq!(*tkn.lexem_ref(), Lexem::KwSafe);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 25;
        assert_eq!(*tkn.lexem_ref(), Lexem::KwUnsafe);
        assert_eq!(*tkn.location_ref(), location);

        tkn = lexer.get().unwrap();
        location.col = 30;
        assert_eq!(*tkn.lexem_ref(), Lexem::KwImpl);
        assert_eq!(*tkn.location_ref(), location);

        assert_eq!(lexer.get(), None);
    }
}
