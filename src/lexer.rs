static FILE_ERROR_MSG: &str = "Cannot open file";

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl Location {
    pub fn new() -> Self {
        Self { row: 1, col: 1 }
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn shift(&mut self) {
        self.col += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    EndOfFile,
    EndOfLine,

    LParen,       // (
    RParen,       // )
    Lcb,          // {
    Rcb,          // }
    Lsb,          // [
    Rsb,          // ]
    Assign,       // =
    Plus,         // +
    AddAssign,    // +=
    Minus,        // -
    SubAssign,    // -=
    Star,         // *
    MulAssign,    // *=
    Slash,        // /
    DivAssign,    // /=
    Percent,      // %
    ModAssign,    // %=
    Eq,           // ==
    Neq,          // !=
    Not,          // !
    Lt,           // <
    Lte,          // <=
    Gt,           // >
    Gte,          // >=
    LShift,       // <<
    RShift,       // >>
    LShiftAssign, // <<=
    RShiftAssign, // >>=
    Stick,        // |
    Ampersand,    // &
    Xor,          // | & ^
    OrAssign,     // |=
    AndAssign,    // &=
    XorAssign,    // ^=
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    Arrow,        // ->

    KwLet,
    KwFunc,
    KwIf,
    KwElse,
    KwDo,
    KwWhile,
    KwFor,
    KwLoop,
    KwContinue,
    KwBreak,
    KwReturn,
    KwModule,
    KwStruct,
    KwAlias,
    KwUse,
    KwExtern,
    KwStatic,
    KwMut,
    KwConst,
    KwOr,
    KwAnd,

    Identifier(String),
    Integer(usize),
    Decimal(f64),

    Unknown,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::EndOfFile => write!(f, "EOF"),
            TokenType::EndOfLine => write!(f, "EOL"),
            TokenType::LParen => write!(f, "("),
            TokenType::RParen => write!(f, ")"),
            TokenType::Lcb => write!(f, "{{"),
            TokenType::Rcb => write!(f, "}}"),
            TokenType::Lsb => write!(f, "["),
            TokenType::Rsb => write!(f, "]"),

            TokenType::Assign => write!(f, "="),
            TokenType::Plus => write!(f, "+"),
            TokenType::AddAssign => write!(f, "+="),
            TokenType::Minus => write!(f, "-"),
            TokenType::SubAssign => write!(f, "-="),
            TokenType::Star => write!(f, "*"),
            TokenType::MulAssign => write!(f, "*="),
            TokenType::Slash => write!(f, "/"),
            TokenType::DivAssign => write!(f, "/="),
            TokenType::Percent => write!(f, "%"),
            TokenType::ModAssign => write!(f, "%="),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::AndAssign => write!(f, "&="),
            TokenType::Stick => write!(f, "|"),
            TokenType::OrAssign => write!(f, "|="),
            TokenType::Xor => write!(f, "^"),
            TokenType::XorAssign => write!(f, "^="),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Colon => write!(f, ":"),
            TokenType::Arrow => write!(f, "->"),

            TokenType::Lt => write!(f, "<"),
            TokenType::Lte => write!(f, "<="),
            TokenType::Gt => write!(f, ">"),
            TokenType::Gte => write!(f, ">="),
            TokenType::LShift => write!(f, "<<"),
            TokenType::LShiftAssign => write!(f, "<<="),
            TokenType::RShift => write!(f, ">>"),
            TokenType::RShiftAssign => write!(f, ">>="),
            TokenType::Eq => write!(f, "=="),
            TokenType::Neq => write!(f, "!="),
            TokenType::Not => write!(f, "!"),

            TokenType::KwLet => write!(f, "let"),
            TokenType::KwFunc => write!(f, "func"),
            TokenType::KwIf => write!(f, "if"),
            TokenType::KwElse => write!(f, "else"),
            TokenType::KwDo => write!(f, "do"),
            TokenType::KwWhile => write!(f, "while"),
            TokenType::KwFor => write!(f, "for"),
            TokenType::KwLoop => write!(f, "loop"),
            TokenType::KwContinue => write!(f, "continue"),
            TokenType::KwBreak => write!(f, "break"),
            TokenType::KwReturn => write!(f, "return"),
            TokenType::KwStruct => write!(f, "struct"),
            TokenType::KwAlias => write!(f, "alias"),
            TokenType::KwModule => write!(f, "module"),
            TokenType::KwUse => write!(f, "use"),
            TokenType::KwExtern => write!(f, "extern"),
            TokenType::KwStatic => write!(f, "static"),
            TokenType::KwMut => write!(f, "mut"),
            TokenType::KwConst => write!(f, "const"),

            TokenType::KwOr => write!(f, "or"),
            TokenType::KwAnd => write!(f, "and"),

            TokenType::Identifier(val) => write!(f, "identifier: \"{}\"", val),
            TokenType::Integer(val) => write!(f, "integer: \"{}\"", val),
            TokenType::Decimal(val) => write!(f, "float: \"{}\"", val),

            TokenType::Unknown => write!(f, "unknown token"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub lexem: TokenType,
    pub location: Location,
}

impl Token {
    pub fn new(lexem: TokenType, location: Location) -> Self {
        Self { lexem, location }
    }

    pub fn get_location(&self) -> Location {
        self.location.clone()
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self.lexem, TokenType::Identifier(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.lexem, TokenType::Integer(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self.lexem, TokenType::Decimal(_))
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: \"{}\"", self.location, self.lexem)
    }
}

pub struct Lexer {
    location: Location,
    next_token: Option<Token>,
    next_char: Option<char>,
    input: Box<dyn std::io::Read>,
    verbose: bool,
    is_eof: bool,
}

impl Lexer {
    pub fn from_file(path: &str, verbose: bool) -> Result<Self, &'static str> {
        let file = std::fs::File::open(path);

        if file.is_err() {
            return Err(FILE_ERROR_MSG);
        }

        Ok(Self {
            location: Location::new(),
            next_token: None,
            next_char: None,
            input: Box::new(file.unwrap()),
            verbose,
            is_eof: false,
        })
    }

    pub fn from_string(src: &'static str, verbose: bool) -> Result<Self, &'static str> {
        Ok(Self {
            location: Location::new(),
            next_token: None,
            next_char: None,
            input: Box::new(src.as_bytes()),
            verbose,
            is_eof: false,
        })
    }

    pub fn collect(&mut self) -> Vec<Token> {
        let verbose = self.verbose;
        self.verbose = false;

        let mut tokens = Vec::<Token>::new();

        while !self.is_eof {
            let tkn = self.get();

            if tkn.lexem == TokenType::Unknown {
                break;
            }

            tokens.push(tkn);
        }

        self.verbose = verbose;

        tokens
    }

    pub fn get(&mut self) -> Token {
        let tkn = self.get_next(false);

        if self.verbose {
            println!("{}", tkn);
        }

        tkn
    }

    pub fn get_singular(&mut self) -> Token {
        let tkn = self.get_next(true);

        if self.verbose {
            println!("{}", tkn);
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
                return;
            }

            self.get_char();
        }
    }

    fn get_next(&mut self, singular: bool) -> Token {
        if self.next_token.is_some() {
            let token = self.next_token.clone().unwrap();
            self.next_token = None;
            return token;
        }

        self.skip_spaces();

        if self.is_eof {
            return Token::new(TokenType::EndOfFile, self.location.clone());
        }

        let next_char = self.peek_char();

        match next_char {
            '\n' => {
                self.location.new_line();
                self.get_char();
                Token::new(TokenType::EndOfLine, self.location.clone())
            }

            '(' => {
                self.get_char();
                Token::new(TokenType::LParen, self.location.clone())
            }

            ')' => {
                self.get_char();
                Token::new(TokenType::RParen, self.location.clone())
            }

            '[' => {
                self.get_char();
                Token::new(TokenType::Lsb, self.location.clone())
            }

            ']' => {
                self.get_char();
                Token::new(TokenType::Rsb, self.location.clone())
            }

            '{' => {
                self.get_char();
                Token::new(TokenType::Lcb, self.location.clone())
            }

            '}' => {
                self.get_char();
                Token::new(TokenType::Rcb, self.location.clone())
            }

            '>' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    return Token::new(TokenType::Gte, self.location.clone());
                }

                if self.peek_char() == '>' && !singular {
                    self.get_char();

                    if self.peek_char() == '=' {
                        self.get_char();
                        return Token::new(TokenType::RShiftAssign, self.location.clone());
                    }

                    return Token::new(TokenType::RShift, self.location.clone());
                }

                Token::new(TokenType::Gt, self.location.clone())
            }

            '<' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    return Token::new(TokenType::Lte, self.location.clone());
                }

                if self.peek_char() == '<' && !singular {
                    self.get_char();

                    if self.peek_char() == '=' {
                        self.get_char();
                        return Token::new(TokenType::LShiftAssign, self.location.clone());
                    }

                    return Token::new(TokenType::LShift, self.location.clone());
                }

                Token::new(TokenType::Lt, self.location.clone())
            }

            '+' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::AddAssign, self.location.clone());
                }

                Token::new(TokenType::Plus, self.location.clone())
            }

            '-' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::SubAssign, self.location.clone());
                }

                if self.peek_char() == '>' && !singular {
                    self.get_char();

                    return Token::new(TokenType::Arrow, self.location.clone());
                }

                Token::new(TokenType::Minus, self.location.clone())
            }

            '/' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::DivAssign, self.location.clone());
                }

                Token::new(TokenType::Slash, self.location.clone())
            }

            '%' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::ModAssign, self.location.clone());
                }

                Token::new(TokenType::Percent, self.location.clone())
            }

            '*' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::MulAssign, self.location.clone());
                }

                Token::new(TokenType::Star, self.location.clone())
            }

            '!' => {
                self.get_char();

                let ch = self.peek_char();

                if ch == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::Neq, self.location.clone());
                }

                Token::new(TokenType::Not, self.location.clone())
            }

            '=' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::Eq, self.location.clone());
                }

                Token::new(TokenType::Assign, self.location.clone())
            }

            '&' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::AndAssign, self.location.clone());
                }

                Token::new(TokenType::Ampersand, self.location.clone())
            }

            '^' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::XorAssign, self.get_location());
                }

                Token::new(TokenType::Xor, self.get_location())
            }

            '|' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::OrAssign, self.get_location());
                }

                Token::new(TokenType::Stick, self.get_location())
            }

            ':' => {
                self.get_char();
                Token::new(TokenType::Colon, self.get_location())
            }

            '.' => {
                self.get_char();
                Token::new(TokenType::Dot, self.get_location())
            }

            ',' => {
                self.get_char();
                Token::new(TokenType::Comma, self.get_location())
            }

            _ => {
                if next_char.is_ascii_digit() {
                    return self.get_numeric_token();
                }

                if next_char.is_ascii_alphabetic() || next_char == '_' {
                    return self.get_string_token();
                }

                Token::new(TokenType::Unknown, self.get_location())
            }
        }
    }

    fn get_numeric_token(&mut self) -> Token {
        let location = self.location.clone();

        let mut text = String::new();
        let mut is_float = false;

        while !self.is_eof && (self.peek_char().is_ascii_digit() || self.peek_char() == '.') {
            if self.peek_char() == '.' {
                if is_float {
                    return Token::new(TokenType::Unknown, location);
                }
                is_float = true;
            }

            text.push(self.get_char());
        }

        if is_float {
            let val = {
                let val = text.parse::<f64>();
                if val.is_err() {
                    return Token::new(TokenType::Unknown, location);
                }
                val.unwrap()
            };

            return Token::new(TokenType::Decimal(val), location);
        }

        let val = {
            let val = text.parse::<usize>();
            if val.is_err() {
                return Token::new(TokenType::Unknown, location);
            }
            val.unwrap()
        };

        Token::new(TokenType::Integer(val), location)
    }

    fn get_string_token(&mut self) -> Token {
        let location = self.location.clone();

        let mut text = String::new();

        while !self.is_eof && (self.peek_char().is_ascii_alphanumeric() || self.peek_char() == '_')
        {
            text.push(self.get_char());
        }

        match &text[..] {
            "module" => Token::new(TokenType::KwModule, location),
            "struct" => Token::new(TokenType::KwStruct, location),
            "let" => Token::new(TokenType::KwLet, location),
            "mut" => Token::new(TokenType::KwMut, location),
            "const" => Token::new(TokenType::KwConst, location),
            "alias" => Token::new(TokenType::KwAlias, location),
            "func" => Token::new(TokenType::KwFunc, location),
            "if" => Token::new(TokenType::KwIf, location),
            "else" => Token::new(TokenType::KwElse, location),
            "loop" => Token::new(TokenType::KwLoop, location),
            "do" => Token::new(TokenType::KwDo, location),
            "while" => Token::new(TokenType::KwWhile, location),
            "for" => Token::new(TokenType::KwFor, location),
            "continue" => Token::new(TokenType::KwContinue, location),
            "break" => Token::new(TokenType::KwBreak, location),
            "return" => Token::new(TokenType::KwReturn, location),
            "extern" => Token::new(TokenType::KwExtern, location),
            "static" => Token::new(TokenType::KwStatic, location),
            "use" => Token::new(TokenType::KwUse, location),
            "or" => Token::new(TokenType::KwOr, location),
            "and" => Token::new(TokenType::KwAnd, location),
            _ => Token::new(TokenType::Identifier(text), location),
        }
    }
}

pub fn dump_tokens(tokens: &[Token]) {
    for tkn in tokens.iter() {
        println!("{}", tkn);
    }
}
