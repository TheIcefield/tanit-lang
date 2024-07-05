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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lexem {
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
    KwDef,
    KwModule,
    KwStruct,
    KwEnum,
    KwAlias,
    KwUse,
    KwExtern,
    KwStatic,
    KwMut,
    KwConst,
    KwAs,

    Identifier(String),
    Integer(String),
    Decimal(String),

    Unknown,
}

impl Lexem {
    pub fn new() -> Self {
        Self::Identifier(String::new())
    }

    pub fn from(v: &str) -> Self {
        Self::Identifier(String::from(v))
    }

    pub fn get_string(&self) -> String {
        match self {
            Self::Identifier(val) | Self::Integer(val) | Self::Decimal(val) => String::from(val),

            _ => String::new(),
        }
    }

    pub fn get_str(&self) -> Option<&str> {
        match self {
            Self::Identifier(val) | Self::Integer(val) | Self::Decimal(val) => Some(val),

            _ => None,
        }
    }

    pub fn get_int(&self) -> Option<usize> {
        match self {
            Self::Integer(val) => match val.parse() {
                Ok(val) => Some(val),
                Err(_) => None,
            },
            _ => None,
        }
    }

    pub fn get_dec(&self) -> Option<f64> {
        match self {
            Self::Decimal(val) => match val.parse() {
                Ok(val) => Some(val),
                Err(_) => None,
            },
            _ => None,
        }
    }
}

impl Default for Lexem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Lexem {
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
            Self::KwDef => write!(f, "def"),
            Self::KwModule => write!(f, "module"),
            Self::KwUse => write!(f, "use"),
            Self::KwExtern => write!(f, "extern"),
            Self::KwStatic => write!(f, "static"),
            Self::KwMut => write!(f, "mut"),
            Self::KwConst => write!(f, "const"),
            Self::KwAs => write!(f, "as"),

            Self::Identifier(val) => write!(f, "{}", val),
            Self::Integer(val) => write!(f, "{}", val),
            Self::Decimal(val) => write!(f, "{}", val),

            Self::Unknown => write!(f, "unknown token"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub lexem: Lexem,
    pub location: Location,
}

impl Token {
    pub fn new(lexem: Lexem, location: Location) -> Self {
        Self { lexem, location }
    }

    pub fn get_location(&self) -> Location {
        self.location.clone()
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self.lexem, Lexem::Identifier(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.lexem, Lexem::Integer(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(self.lexem, Lexem::Decimal(_))
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: \"{}\"", self.location, self.lexem)
    }
}

pub struct Lexer {
    path: String,
    location: Location,
    next_token: Option<Token>,
    next_char: Option<char>,
    input: Box<dyn std::io::Read>,
    pub ignores_nl: bool,
    pub verbose: bool,
    is_eof: bool,
}

impl Lexer {
    pub fn from_file(path: &str, verbose: bool) -> Result<Self, &'static str> {
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
            verbose,
            is_eof: false,
        })
    }

    pub fn from_text(src: &'static str, verbose: bool) -> Result<Self, &'static str> {
        Ok(Self {
            path: String::new(),
            location: Location::new(),
            next_token: None,
            next_char: None,
            input: Box::new(src.as_bytes()),
            ignores_nl: true,
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

            if tkn.lexem == Lexem::Unknown {
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

    pub fn get_path(&self) -> Result<String, &'static str> {
        if self.path.is_empty() {
            return Err("Lexer input stream is not a file");
        }

        Ok(self.path.clone())
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
            return Token::new(Lexem::EndOfFile, self.location.clone());
        }

        let next_char = self.peek_char();

        match next_char {
            '\n' => {
                self.get_char();
                Token::new(Lexem::EndOfLine, self.location.clone())
            }

            '(' => {
                self.get_char();
                Token::new(Lexem::LParen, self.location.clone())
            }

            ')' => {
                self.get_char();
                Token::new(Lexem::RParen, self.location.clone())
            }

            '[' => {
                self.get_char();
                Token::new(Lexem::Lsb, self.location.clone())
            }

            ']' => {
                self.get_char();
                Token::new(Lexem::Rsb, self.location.clone())
            }

            '{' => {
                self.get_char();
                Token::new(Lexem::Lcb, self.location.clone())
            }

            '}' => {
                self.get_char();
                Token::new(Lexem::Rcb, self.location.clone())
            }

            '>' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    return Token::new(Lexem::Gte, self.location.clone());
                }

                if self.peek_char() == '>' && !singular {
                    self.get_char();

                    if self.peek_char() == '=' {
                        self.get_char();
                        return Token::new(Lexem::RShiftAssign, self.location.clone());
                    }

                    return Token::new(Lexem::RShift, self.location.clone());
                }

                Token::new(Lexem::Gt, self.location.clone())
            }

            '<' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();
                    return Token::new(Lexem::Lte, self.location.clone());
                }

                if self.peek_char() == '<' && !singular {
                    self.get_char();

                    if self.peek_char() == '=' {
                        self.get_char();
                        return Token::new(Lexem::LShiftAssign, self.location.clone());
                    }

                    return Token::new(Lexem::LShift, self.location.clone());
                }

                Token::new(Lexem::Lt, self.location.clone())
            }

            '+' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::AddAssign, self.location.clone());
                }

                Token::new(Lexem::Plus, self.location.clone())
            }

            '-' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::SubAssign, self.location.clone());
                }

                if self.peek_char() == '>' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Arrow, self.location.clone());
                }

                Token::new(Lexem::Minus, self.location.clone())
            }

            '/' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::DivAssign, self.location.clone());
                }

                Token::new(Lexem::Slash, self.location.clone())
            }

            '%' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::ModAssign, self.location.clone());
                }

                Token::new(Lexem::Percent, self.location.clone())
            }

            '*' => {
                self.get_char();

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::MulAssign, self.location.clone());
                }

                Token::new(Lexem::Star, self.location.clone())
            }

            '!' => {
                self.get_char();

                let ch = self.peek_char();

                if ch == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Neq, self.location.clone());
                }

                Token::new(Lexem::Not, self.location.clone())
            }

            '=' => {
                self.get_char();
                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::Eq, self.location.clone());
                }

                Token::new(Lexem::Assign, self.location.clone())
            }

            '&' => {
                self.get_char();
                if self.peek_char() == '&' && !singular {
                    self.get_char();

                    return Token::new(Lexem::AndAssign, self.location.clone());
                }

                if self.peek_char() == '=' && !singular {
                    self.get_char();

                    return Token::new(Lexem::And, self.location.clone());
                }

                Token::new(Lexem::Ampersand, self.location.clone())
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
        let location = self.location.clone();

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
        let location = self.location.clone();

        let mut text = String::new();

        while !self.is_eof && (self.peek_char().is_ascii_alphanumeric() || self.peek_char() == '_')
        {
            text.push(self.get_char());
        }

        match &text[..] {
            "def" => Token::new(Lexem::KwDef, location),
            "module" => Token::new(Lexem::KwModule, location),
            "struct" => Token::new(Lexem::KwStruct, location),
            "enum" => Token::new(Lexem::KwEnum, location),
            "let" => Token::new(Lexem::KwLet, location),
            "mut" => Token::new(Lexem::KwMut, location),
            "const" => Token::new(Lexem::KwConst, location),
            "alias" => Token::new(Lexem::KwAlias, location),
            "func" => Token::new(Lexem::KwFunc, location),
            "if" => Token::new(Lexem::KwIf, location),
            "else" => Token::new(Lexem::KwElse, location),
            "loop" => Token::new(Lexem::KwLoop, location),
            "do" => Token::new(Lexem::KwDo, location),
            "while" => Token::new(Lexem::KwWhile, location),
            "for" => Token::new(Lexem::KwFor, location),
            "continue" => Token::new(Lexem::KwContinue, location),
            "break" => Token::new(Lexem::KwBreak, location),
            "return" => Token::new(Lexem::KwReturn, location),
            "extern" => Token::new(Lexem::KwExtern, location),
            "static" => Token::new(Lexem::KwStatic, location),
            "use" => Token::new(Lexem::KwUse, location),
            "as" => Token::new(Lexem::KwAs, location),
            _ => Token::new(Lexem::Identifier(text), location),
        }
    }
}

pub fn dump_tokens(tokens: &[Token]) {
    for tkn in tokens.iter() {
        println!("{}", tkn);
    }
}
