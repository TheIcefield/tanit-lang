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
    Or,           // ||
    Ampersand,    // &
    And,          // &&
    Xor,          // ^
    OrAssign,     // |=
    AndAssign,    // &=
    XorAssign,    // ^=
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    Dcolon,       // ::
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
    KwEnum,
    KwAlias,
    KwUse,
    KwExtern,
    KwStatic,
    KwMut,
    KwConst,

    Identifier(String),
    Integer(usize),
    Decimal(f64),

    Unknown,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfFile => write!(f, "EOF"),
            Self::EndOfLine => write!(f, "EOL"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Lcb => write!(f, "{{"),
            Self::Rcb => write!(f, "}}"),
            Self::Lsb => write!(f, "["),
            Self::Rsb => write!(f, "]"),

            Self::Assign => write!(f, "="),
            Self::Plus => write!(f, "+"),
            Self::AddAssign => write!(f, "+="),
            Self::Minus => write!(f, "-"),
            Self::SubAssign => write!(f, "-="),
            Self::Star => write!(f, "*"),
            Self::MulAssign => write!(f, "*="),
            Self::Slash => write!(f, "/"),
            Self::DivAssign => write!(f, "/="),
            Self::Percent => write!(f, "%"),
            Self::ModAssign => write!(f, "%="),
            Self::Ampersand => write!(f, "&"),
            Self::And => write!(f, "&&"),
            Self::AndAssign => write!(f, "&="),
            Self::Stick => write!(f, "|"),
            Self::Or => write!(f, "||"),
            Self::OrAssign => write!(f, "|="),
            Self::Xor => write!(f, "^"),
            Self::XorAssign => write!(f, "^="),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Colon => write!(f, ":"),
            Self::Dcolon => write!(f, "::"),
            Self::Arrow => write!(f, "->"),

            Self::Lt => write!(f, "<"),
            Self::Lte => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Gte => write!(f, ">="),
            Self::LShift => write!(f, "<<"),
            Self::LShiftAssign => write!(f, "<<="),
            Self::RShift => write!(f, ">>"),
            Self::RShiftAssign => write!(f, ">>="),
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "!="),
            Self::Not => write!(f, "!"),

            Self::KwLet => write!(f, "let"),
            Self::KwFunc => write!(f, "func"),
            Self::KwIf => write!(f, "if"),
            Self::KwElse => write!(f, "else"),
            Self::KwDo => write!(f, "do"),
            Self::KwWhile => write!(f, "while"),
            Self::KwFor => write!(f, "for"),
            Self::KwLoop => write!(f, "loop"),
            Self::KwContinue => write!(f, "continue"),
            Self::KwBreak => write!(f, "break"),
            Self::KwReturn => write!(f, "return"),
            Self::KwStruct => write!(f, "struct"),
            Self::KwEnum => write!(f, "enum"),
            Self::KwAlias => write!(f, "alias"),
            Self::KwModule => write!(f, "module"),
            Self::KwUse => write!(f, "use"),
            Self::KwExtern => write!(f, "extern"),
            Self::KwStatic => write!(f, "static"),
            Self::KwMut => write!(f, "mut"),
            Self::KwConst => write!(f, "const"),

            Self::Identifier(val) => write!(f, "identifier: \"{}\"", val),
            Self::Integer(val) => write!(f, "integer: \"{}\"", val),
            Self::Decimal(val) => write!(f, "float: \"{}\"", val),

            Self::Unknown => write!(f, "unknown token"),
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
                if self.peek_char() == '&' && !singular {
                    self.get_char();

                    return Token::new(TokenType::AndAssign, self.location.clone());
                }

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(TokenType::And, self.location.clone());
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

                if self.peek_char() == '|' && !singular {
                    self.get_char();

                    return Token::new(TokenType::Or, self.get_location());
                }

                Token::new(TokenType::Stick, self.get_location())
            }

            ':' => {
                self.get_char();
                if self.peek_char() == ':' && !singular {
                    self.get_char();

                    return Token::new(TokenType::Dcolon, self.get_location());
                }

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
            "enum" => Token::new(TokenType::KwEnum, location),
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
            _ => Token::new(TokenType::Identifier(text), location),
        }
    }
}

pub fn dump_tokens(tokens: &[Token]) {
    for tkn in tokens.iter() {
        println!("{}", tkn);
    }
}
