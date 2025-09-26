pub mod location;
pub mod token;

use std::path::{Path, PathBuf};

use location::Location;
use token::{Lexem, Token};

pub struct Lexer {
    path: PathBuf,
    location: Location,
    next_token: Option<Token>,
    next_char: Option<char>,
    input: Box<dyn std::io::Read>,
    pub ignores_nl: bool,
    pub verbose_tokens: bool,
    is_eof: bool,
}

impl Lexer {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path);

        if let Err(err) = file {
            return Err(format!("Failed to open {path:?}: {err}"));
        }

        Ok(Self {
            path: path.to_path_buf(),
            location: Location::new(path),
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
            path: PathBuf::new(),
            location: Location::new(&PathBuf::new()),
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
        self.location.clone()
    }

    pub fn get_path(&self) -> &Path {
        if self.path.as_os_str().is_empty() {
            panic!("Lexer input stream is not a file");
        }

        self.path.as_path()
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

        let location = self.get_location();

        if self.is_eof {
            return Token::new(Lexem::EndOfFile, location);
        }

        let next_char = self.peek_char();

        let lexem = match next_char {
            '\n' => {
                self.get_char();
                Lexem::EndOfLine
            }
            '(' => {
                self.get_char();
                Lexem::LParen
            }
            ')' => {
                self.get_char();
                Lexem::RParen
            }
            '[' => {
                self.get_char();
                Lexem::Lsb
            }
            ']' => {
                self.get_char();
                Lexem::Rsb
            }
            '{' => {
                self.get_char();
                Lexem::Lcb
            }
            '}' => {
                self.get_char();
                Lexem::Rcb
            }
            '>' => {
                self.get_char();
                let mut lexem = Lexem::Gt;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::Gte;
                } else if self.peek_char() == '>' && !singular {
                    self.get_char();
                    lexem = Lexem::RShift;

                    if self.peek_char() == '=' {
                        self.get_char();
                        lexem = Lexem::RShiftAssign;
                    }
                }

                lexem
            }
            '<' => {
                self.get_char();
                let mut lexem = Lexem::Lt;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::Lte;
                } else if self.peek_char() == '<' && !singular {
                    self.get_char();
                    lexem = Lexem::LShift;

                    if self.peek_char() == '=' {
                        self.get_char();
                        lexem = Lexem::LShiftAssign;
                    }
                }

                lexem
            }
            '+' => {
                self.get_char();
                let mut lexem = Lexem::Plus;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::AddAssign;
                }

                lexem
            }
            '-' => {
                self.get_char();
                let mut lexem = Lexem::Minus;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::SubAssign;
                }

                lexem
            }
            '/' => {
                self.get_char();
                let mut lexem = Lexem::Slash;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::DivAssign;
                }

                lexem
            }
            '%' => {
                self.get_char();
                let mut lexem = Lexem::Percent;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::ModAssign;
                }

                lexem
            }
            '*' => {
                self.get_char();
                let mut lexem = Lexem::Star;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::MulAssign;
                }

                lexem
            }
            '!' => {
                self.get_char();
                let mut lexem = Lexem::Not;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::Neq;
                }

                lexem
            }
            '=' => {
                self.get_char();
                let mut lexem = Lexem::Assign;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::Eq;
                }

                lexem
            }
            '&' => {
                self.get_char();
                let mut lexem = Lexem::Ampersand;

                if self.peek_char() == '&' && !singular {
                    self.get_char();
                    lexem = Lexem::And;
                } else if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::AndAssign;
                }

                lexem
            }
            '^' => {
                self.get_char();
                let mut lexem = Lexem::Xor;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::XorAssign;
                }

                lexem
            }
            '|' => {
                self.get_char();
                let mut lexem = Lexem::Stick;

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    lexem = Lexem::OrAssign;
                } else if self.peek_char() == '|' && !singular {
                    self.get_char();
                    lexem = Lexem::Or;
                }

                lexem
            }
            ':' => {
                self.get_char();
                let mut lexem = Lexem::Colon;

                if self.peek_char() == ':' && !singular {
                    self.get_char();
                    lexem = Lexem::Dcolon;
                }

                lexem
            }
            '.' => {
                self.get_char();
                Lexem::Dot
            }
            ',' => {
                self.get_char();
                Lexem::Comma
            }
            _ if '\"' == next_char => self.get_text_lexem(),
            _ if next_char.is_ascii_digit() => self.get_numeric_lexem(),
            _ if next_char.is_ascii_alphabetic() || '_' == next_char => self.get_string_lexem(),
            _ => Lexem::Unknown,
        };

        Token::new(lexem, location)
    }

    fn get_numeric_lexem(&mut self) -> Lexem {
        let mut text = String::new();
        let mut is_float = false;

        while !self.is_eof && (self.peek_char().is_ascii_digit() || self.peek_char() == '.') {
            if self.peek_char() == '.' {
                if is_float {
                    return Lexem::Unknown;
                }
                is_float = true;
            }

            text.push(self.get_char());
        }

        if is_float {
            return Lexem::Decimal(text);
        }

        Lexem::Integer(text)
    }

    fn get_string_lexem(&mut self) -> Lexem {
        let mut text = String::new();

        while !self.is_eof && (self.peek_char().is_ascii_alphanumeric() || self.peek_char() == '_')
        {
            text.push(self.get_char());
        }

        match &text[..] {
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
        }
    }

    fn get_text_lexem(&mut self) -> Lexem {
        let mut text = String::new();

        self.get_char();

        while !self.is_eof && self.peek_char().is_ascii() {
            if self.peek_char() == '\"' {
                self.get_char();
                break;
            }

            text.push(self.get_char());
        }

        Lexem::Text(text)
    }
}

/*
+ hello func var 65 -= <<\n
 struct alpha safe unsafe impl
 */

#[test]
fn lexer_with_ignore_test() {
    const SRC_TEXT: &str = "+ hello func var 65 -= <<\n struct alpha safe unsafe impl";

    let mut lexer = Lexer::from_text(SRC_TEXT).unwrap();
    lexer.ignores_nl = true;

    let mut tkn = lexer.get();
    let mut location = Location::default();

    location.col = 2;
    assert_eq!(tkn.lexem, Lexem::Plus);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 4;
    assert_eq!(tkn.lexem, Lexem::identifier("hello"));
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 10;
    assert_eq!(tkn.lexem, Lexem::KwFunc);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 15;
    assert_eq!(tkn.lexem, Lexem::KwVar);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 19;
    assert_eq!(tkn.lexem, Lexem::integer(65));
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 22;
    assert_eq!(tkn.lexem, Lexem::SubAssign);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 25;
    assert_eq!(tkn.lexem, Lexem::LShift);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.row = 2;
    location.col = 3;
    assert_eq!(tkn.lexem, Lexem::KwStruct);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 10;
    assert_eq!(tkn.lexem, Lexem::identifier("alpha"));
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 16;
    assert_eq!(tkn.lexem, Lexem::KwSafe);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 21;
    assert_eq!(tkn.lexem, Lexem::KwUnsafe);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 28;
    assert_eq!(tkn.lexem, Lexem::KwImpl);
    assert_eq!(tkn.location, location);
}

#[test]
fn lexer_without_test() {
    const SRC_TEXT: &str = "+ hello func var 65 -= <<\n struct alpha safe unsafe impl";

    let mut lexer = Lexer::from_text(SRC_TEXT).unwrap();
    lexer.ignores_nl = false;

    let mut tkn = lexer.get();
    let mut location = Location::default();

    location.col = 2;
    assert_eq!(tkn.lexem, Lexem::Plus);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 4;
    assert_eq!(tkn.lexem, Lexem::identifier("hello"));
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 10;
    assert_eq!(tkn.lexem, Lexem::KwFunc);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 15;
    assert_eq!(tkn.lexem, Lexem::KwVar);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 19;
    assert_eq!(tkn.lexem, Lexem::integer(65));
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 22;
    assert_eq!(tkn.lexem, Lexem::SubAssign);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 25;
    assert_eq!(tkn.lexem, Lexem::LShift);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.row = 2;
    location.col = 1;
    assert_eq!(tkn.lexem, Lexem::EndOfLine);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 3;
    assert_eq!(tkn.lexem, Lexem::KwStruct);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 10;
    assert_eq!(tkn.lexem, Lexem::identifier("alpha"));
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 16;
    assert_eq!(tkn.lexem, Lexem::KwSafe);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 21;
    assert_eq!(tkn.lexem, Lexem::KwUnsafe);
    assert_eq!(tkn.location, location);

    tkn = lexer.get();
    location.col = 28;
    assert_eq!(tkn.lexem, Lexem::KwImpl);
    assert_eq!(tkn.location, location);
}
