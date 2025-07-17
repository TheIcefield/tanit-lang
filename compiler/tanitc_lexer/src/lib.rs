pub mod location;
pub mod token;

use location::Location;
use token::{Lexem, Token};

static FILE_ERROR_MSG: &str = "Cannot open file";

pub struct Lexer {
    path: String,
    location: Location,
    next_token: Option<Token>,
    next_char: Option<char>,
    input: Box<dyn std::io::Read>,
    pub ignores_nl: bool,
    pub verbose_tokens: bool,
    is_eof: bool,
}

impl Lexer {
    pub fn from_file(path: &str) -> Result<Self, &'static str> {
        let file = std::fs::File::open(path);

        if file.is_err() {
            return Err(FILE_ERROR_MSG);
        }

        Ok(Self {
            path: path.to_string(),
            location: Location::new(),
            next_token: None,
            next_char: None,
            input: Box::new(file.unwrap()),
            ignores_nl: true,
            verbose_tokens: false,
            is_eof: false,
        })
    }

    pub fn from_text(src: &'static str) -> Result<Self, &'static str> {
        Ok(Self {
            path: String::new(),
            location: Location::new(),
            next_token: None,
            next_char: None,
            input: Box::new(src.as_bytes()),
            ignores_nl: true,
            verbose_tokens: false,
            is_eof: false,
        })
    }

    pub fn get(&mut self) -> Token {
        let tkn = self.get_next(false);

        if self.verbose_tokens {
            println!("{tkn}");
        }

        tkn
    }

    pub fn get_singular(&mut self) -> Token {
        let tkn = self.get_next(true);

        if self.verbose_tokens {
            println!("{tkn}");
        }

        tkn
    }

    pub fn peek(&mut self) -> Token {
        if self.next_token.is_some() {
            let tkn = self.next_token.clone().unwrap();
            return tkn;
        }

        self.next_token = Some(self.get_next(false));

        self.next_token.clone().unwrap()
    }

    pub fn peek_singular(&mut self) -> Token {
        if self.next_token.is_some() {
            let tkn = self.next_token.clone().unwrap();
            return tkn;
        }

        self.next_token = Some(self.get_next(true));

        self.next_token.clone().unwrap()
    }

    pub fn get_location(&self) -> Location {
        self.location
    }

    pub fn get_path(&self) -> String {
        if self.path.is_empty() {
            panic!("Lexer input stream is not a file");
        }

        self.path.clone()
    }
}

/* Private methods */
impl Lexer {
    fn peek_char(&mut self) -> char {
        if self.next_char.is_some() {
            return self.next_char.unwrap();
        }

        self.next_char = Some(self.get_char());

        self.next_char.unwrap()
    }

    fn get_char(&mut self) -> char {
        if self.next_char.is_some() {
            let ch = self.next_char.unwrap();
            self.next_char = None;
            return ch;
        }

        let mut buf: [u8; 1] = [0];

        let res = self.input.read(&mut buf);

        if res.is_err() || buf[0] == 0 {
            self.is_eof = true;
        } else {
            self.location.shift();
        }

        self.next_char = Some(buf[0] as char);
        self.next_char.unwrap()
    }

    fn skip_spaces(&mut self) {
        while self.peek_char().is_ascii_whitespace() {
            if self.peek_char() == '\n' {
                self.location.new_line();

                if !self.ignores_nl {
                    return;
                }
            }

            self.get_char();
        }
    }

    fn skip_comment(&mut self) {
        if self.peek_char() == '#' {
            let old_opt = self.ignores_nl;
            self.ignores_nl = false;

            while self.get_char() != '\n' {}

            self.location.new_line();

            self.ignores_nl = old_opt;
        }
    }

    fn get_next(&mut self, singular: bool) -> Token {
        if self.next_token.is_some() {
            let token = self.next_token.clone().unwrap();
            self.next_token = None;
            return token;
        }

        loop {
            self.skip_spaces();
            self.skip_comment();

            let ch = self.peek_char();

            if (ch != '#' && !ch.is_ascii_whitespace()) || ch == '\n' {
                break;
            }
        }

        if self.is_eof {
            return Token::new(Lexem::EndOfFile, self.location);
        }

        let next_char = self.peek_char();

        match next_char {
            '\n' => {
                self.get_char();
                Token::new(Lexem::EndOfLine, self.location)
            }

            '(' => {
                self.get_char();
                Token::new(Lexem::LParen, self.location)
            }

            ')' => {
                self.get_char();
                Token::new(Lexem::RParen, self.location)
            }

            '[' => {
                self.get_char();
                Token::new(Lexem::Lsb, self.location)
            }

            ']' => {
                self.get_char();
                Token::new(Lexem::Rsb, self.location)
            }

            '{' => {
                self.get_char();
                Token::new(Lexem::Lcb, self.location)
            }

            '}' => {
                self.get_char();
                Token::new(Lexem::Rcb, self.location)
            }

            '>' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    return Token::new(Lexem::Gte, self.location);
                }

                if self.peek_char() == '>' && !singular {
                    self.get_char();

                    if self.peek_char() == '=' {
                        self.get_char();
                        return Token::new(Lexem::RShiftAssign, self.location);
                    }

                    return Token::new(Lexem::RShift, self.location);
                }

                Token::new(Lexem::Gt, self.location)
            }

            '<' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    return Token::new(Lexem::Lte, self.location);
                }

                if self.peek_char() == '<' && !singular {
                    self.get_char();

                    if self.peek_char() == '=' {
                        self.get_char();
                        return Token::new(Lexem::LShiftAssign, self.location);
                    }

                    return Token::new(Lexem::LShift, self.location);
                }

                Token::new(Lexem::Lt, self.location)
            }

            '+' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::AddAssign, self.location);
                }

                Token::new(Lexem::Plus, self.location)
            }

            '-' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::SubAssign, self.location);
                }

                Token::new(Lexem::Minus, self.location)
            }

            '/' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::DivAssign, self.location);
                }

                Token::new(Lexem::Slash, self.location)
            }

            '%' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::ModAssign, self.location);
                }

                Token::new(Lexem::Percent, self.location)
            }

            '*' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::MulAssign, self.location);
                }

                Token::new(Lexem::Star, self.location)
            }

            '!' => {
                self.get_char();

                let ch = self.peek_char();

                if ch == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Neq, self.location);
                }

                Token::new(Lexem::Not, self.location)
            }

            '=' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Eq, self.location);
                }

                Token::new(Lexem::Assign, self.location)
            }

            '&' => {
                self.get_char();
                if self.peek_char() == '&' && !singular {
                    self.get_char();

                    return Token::new(Lexem::AndAssign, self.location);
                }

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::And, self.location);
                }

                Token::new(Lexem::Ampersand, self.location)
            }

            '^' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::XorAssign, self.get_location());
                }

                Token::new(Lexem::Xor, self.get_location())
            }

            '|' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::OrAssign, self.get_location());
                }

                if self.peek_char() == '|' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Or, self.get_location());
                }

                Token::new(Lexem::Stick, self.get_location())
            }

            ':' => {
                self.get_char();
                if self.peek_char() == ':' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Dcolon, self.get_location());
                }

                Token::new(Lexem::Colon, self.get_location())
            }

            '.' => {
                self.get_char();
                Token::new(Lexem::Dot, self.get_location())
            }

            ',' => {
                self.get_char();
                Token::new(Lexem::Comma, self.get_location())
            }

            _ => {
                if next_char == '\"' {
                    return self.get_text_token();
                }

                if next_char.is_ascii_digit() {
                    return self.get_numeric_token();
                }

                if next_char.is_ascii_alphabetic() || next_char == '_' {
                    return self.get_string_token();
                }

                Token::new(Lexem::Unknown, self.get_location())
            }
        }
    }

    fn get_numeric_token(&mut self) -> Token {
        let location = self.location;

        let mut text = String::new();
        let mut is_float = false;

        while !self.is_eof && (self.peek_char().is_ascii_digit() || self.peek_char() == '.') {
            if self.peek_char() == '.' {
                if is_float {
                    return Token::new(Lexem::Unknown, location);
                }
                is_float = true;
            }

            text.push(self.get_char());
        }

        if is_float {
            return Token::new(Lexem::Decimal(text), location);
        }

        Token::new(Lexem::Integer(text), location)
    }

    fn get_string_token(&mut self) -> Token {
        let location = self.location;

        let mut text = String::new();

        while !self.is_eof && (self.peek_char().is_ascii_alphanumeric() || self.peek_char() == '_')
        {
            text.push(self.get_char());
        }

        let lexem = match &text[..] {
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
        };

        Token::new(lexem, location)
    }

    fn get_text_token(&mut self) -> Token {
        let location = self.location;

        let mut text = String::new();

        self.get_char();

        while !self.is_eof && self.peek_char().is_ascii() {
            if self.peek_char() == '\"' {
                self.get_char();
                break;
            }

            text.push(self.get_char());
        }

        Token::new(Lexem::Text(text), location)
    }
}

#[test]
fn lexer_test() {
    const SRC_TEXT: &str = "hello func var + 65 -= <<\n struct alpha safe unsafe impl";

    let mut lexer = Lexer::from_text(SRC_TEXT).unwrap();

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("hello".to_string()),
            Location { row: 1, col: 2 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwFunc, Location { row: 1, col: 8 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwVar, Location { row: 1, col: 13 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::Plus, Location { row: 1, col: 18 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Integer("65".to_string()),
            Location { row: 1, col: 19 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::SubAssign, Location { row: 1, col: 23 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::LShift, Location { row: 1, col: 27 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwStruct, Location { row: 2, col: 3 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("alpha".to_string()),
            Location { row: 2, col: 10 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwSafe, Location { row: 2, col: 16 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwUnsafe, Location { row: 2, col: 21 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwImpl, Location { row: 2, col: 28 })
    );
}

#[test]
fn lexer_without_ignore_test() {
    const SRC_TEXT: &str = "hello func var + 65 -= <<\n struct alpha";

    let mut lexer = Lexer::from_text(SRC_TEXT).unwrap();

    lexer.ignores_nl = false;

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("hello".to_string()),
            Location { row: 1, col: 2 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwFunc, Location { row: 1, col: 8 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwVar, Location { row: 1, col: 13 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::Plus, Location { row: 1, col: 18 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Integer("65".to_string()),
            Location { row: 1, col: 19 }
        )
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::SubAssign, Location { row: 1, col: 23 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::LShift, Location { row: 1, col: 27 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::EndOfLine, Location { row: 2, col: 1 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(Lexem::KwStruct, Location { row: 2, col: 3 })
    );

    assert_eq!(
        lexer.get(),
        Token::new(
            Lexem::Identifier("alpha".to_string()),
            Location { row: 2, col: 10 }
        )
    );
}
