use crate::analyzer::SymbolData;
use crate::ast::{expressions::Expression, identifiers::Identifier, Ast, IAst, Stream};
use crate::error_listener::MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR;
use crate::lexer::Lexem;
use crate::parser::{put_intent, Parser};

use std::io::Write;
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub enum Type {
    Ref {
        is_mut: bool,
        ref_to: Box<Type>,
    },
    Ptr {
        is_mut: bool,
        ptr_to: Box<Type>,
    },
    Tuple {
        components: Vec<Type>,
    },
    Array {
        size: Option<Box<Ast>>,
        value_type: Box<Type>,
    },
    Template {
        identifier: String,
        arguments: Vec<Type>,
    },
    Custom(String),
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Str,
}

impl Type {
    pub fn new() -> Self {
        Self::Tuple {
            components: Vec::new(),
        }
    }

    pub fn from_id(id: &Identifier) -> Self {
        match id {
            Identifier::Common(id) => Self::from_str(id).unwrap(),
            Identifier::Complex(..) => unimplemented!("creation type by complex id"),
        }
    }

    pub fn is_common(&self) -> bool {
        matches!(
            self,
            Self::Bool
                | Self::F32
                | Self::F64
                | Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::I128
                | Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
                | Self::U128
        )
    }

    pub fn parse(parser: &mut Parser) -> Result<Self, &'static str> {
        let next = parser.peek_token();

        if parser.peek_token().lexem == Lexem::Ampersand {
            let mut is_mut = false;
            parser.get_token();

            if matches!(parser.peek_token().lexem, Lexem::KwMut) {
                is_mut = true;
                parser.get_token();
            }

            return Ok(Type::Ref {
                is_mut,
                ref_to: Box::new(Self::parse(parser)?),
            });
        }

        if next.lexem == Lexem::Star {
            let mut is_mut = false;
            parser.get_token();

            if matches!(parser.peek_token().lexem, Lexem::KwMut) {
                is_mut = true;
                parser.get_token();
            }

            return Ok(Type::Ptr {
                is_mut,
                ptr_to: Box::new(Self::parse(parser)?),
            });
        }

        if next.lexem == Lexem::LParen {
            return Self::parse_tuple_def(parser);
        }

        if next.lexem == Lexem::Lsb {
            return Self::parse_array_def(parser);
        }

        let identifier = parser.consume_identifier()?.get_string();

        match &identifier[..] {
            "bool" => return Ok(Type::Bool),
            "i8" => return Ok(Type::I8),
            "i16" => return Ok(Type::I16),
            "i32" => return Ok(Type::I32),
            "i64" => return Ok(Type::I64),
            "i128" => return Ok(Type::I128),
            "u8" => return Ok(Type::U8),
            "u16" => return Ok(Type::U16),
            "u32" => return Ok(Type::U32),
            "u64" => return Ok(Type::U64),
            "u128" => return Ok(Type::U128),
            "f32" => return Ok(Type::F32),
            "f64" => return Ok(Type::F64),
            "str" => return Ok(Type::Str),
            _ => {}
        }

        if parser.peek_singular().lexem == Lexem::Lt {
            let arguments = Self::parse_template_args(parser)?;

            return Ok(Type::Template {
                identifier,
                arguments,
            });
        }

        Ok(Type::Custom(identifier))
    }

    pub fn parse_tuple_def(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(Lexem::LParen)?;

        let mut children = Vec::<Type>::new();
        loop {
            if parser.peek_token().lexem == Lexem::RParen {
                break;
            }

            let child = Self::parse(parser)?;
            children.push(child);

            if parser.peek_token().lexem == Lexem::Comma {
                parser.get_token();
                continue;
            }
        }

        parser.consume_token(Lexem::RParen)?;

        Ok(Type::Tuple {
            components: children,
        })
    }

    pub fn parse_array_def(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(Lexem::Lsb)?;

        let mut size: Option<Box<Ast>> = None;

        let value_type = Box::new(Self::parse(parser)?);

        if parser.peek_token().lexem == Lexem::Colon {
            parser.get_token();

            size = Some(Box::new(Expression::parse(parser)?));
        }

        parser.consume_token(Lexem::Rsb)?;

        Ok(Type::Array { size, value_type })
    }

    pub fn parse_template_args(parser: &mut Parser) -> Result<Vec<Self>, &'static str> {
        parser.consume_token(Lexem::Lt)?;

        let mut children = Vec::<Type>::new();
        loop {
            let child = Self::parse(parser)?;
            children.push(child);

            let next = parser.peek_singular();
            if next.lexem == Lexem::Gt {
                break;
            } else {
                parser.consume_token(Lexem::Comma)?;
            }
        }

        parser.get_singular();

        Ok(children)
    }

    fn get_c_type(&self) -> String {
        match self {
            Self::Bool | Self::U8 => "unsigned char",
            Self::U16 => "unsigned short",
            Self::U32 => "unsigned int",
            Self::U64 => "unsigned long",
            Self::U128 => "unsigned long long",
            Self::I8 => "unsigned int",
            Self::I16 => "signed short",
            Self::I32 => "signed int",
            Self::I64 => "signed long",
            Self::I128 => "signed long long",
            Self::F32 => "float",
            Self::F64 => "double",
            _ => unimplemented!(),
        }
        .to_string()
    }
}

impl std::str::FromStr for Type {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bool" => Ok(Type::Bool),
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "i128" => Ok(Type::I128),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "u128" => Ok(Type::U128),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "str" => Ok(Type::Str),
            _ => Ok(Type::Custom(s.to_string())),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Alias {
    pub identifier: Identifier,
    pub value: Type,
}

impl Alias {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(Lexem::KwAlias)?;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        parser.consume_token(Lexem::Assign)?;

        let value = Type::parse(parser)?;

        Ok(Ast::AliasDef {
            node: Alias { identifier, value },
        })
    }
}

impl IAst for Type {
    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        unreachable!("Type.analyze() shouln't have been invocked");
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Ref { is_mut, ref_to } => {
                writeln!(stream, "{}<ref is_mut=\"{}\">", put_intent(intent), is_mut)?;

                ref_to.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</ref>", put_intent(intent))?;
            }

            Self::Ptr { is_mut, ptr_to } => {
                writeln!(stream, "{}<ptr is_mut=\"{}\">", put_intent(intent), is_mut)?;

                ptr_to.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</ptr>", put_intent(intent))?;
            }

            Self::Tuple { components } => {
                writeln!(stream, "{}<tuple>", put_intent(intent))?;

                for comp in components.iter() {
                    comp.traverse(stream, intent + 1)?;
                }

                writeln!(stream, "{}</tuple>", put_intent(intent))?;
            }

            Self::Array { size, value_type } => {
                writeln!(stream, "{}<array>", put_intent(intent))?;

                if let Some(size) = size {
                    writeln!(stream, "{}<size>", put_intent(intent + 1))?;

                    size.traverse(stream, intent + 2)?;

                    writeln!(stream, "{}</size>", put_intent(intent + 1))?;
                }

                value_type.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</array>", put_intent(intent))?;
            }

            Self::Template {
                identifier,
                arguments,
            } => {
                writeln!(
                    stream,
                    "{}<type name=\"{}\">",
                    put_intent(intent),
                    identifier
                )?;

                for arg in arguments.iter() {
                    arg.traverse(stream, intent + 1)?;
                }

                writeln!(stream, "{}</type>", put_intent(intent))?;
            }

            Self::Custom(id) => {
                writeln!(stream, "{}<type name=\"{}\"/>", put_intent(intent), id)?;
            }

            Self::Bool => writeln!(stream, "{}<type name=\"bool\"/>", put_intent(intent))?,
            Self::I8 => writeln!(stream, "{}<type name=\"i8\"/>", put_intent(intent))?,
            Self::I16 => writeln!(stream, "{}<type name=\"i16\"/>", put_intent(intent))?,
            Self::I32 => writeln!(stream, "{}<type name=\"i32\"/>", put_intent(intent))?,
            Self::I64 => writeln!(stream, "{}<type name=\"i64\"/>", put_intent(intent))?,
            Self::I128 => writeln!(stream, "{}<type name=\"i128\"/>", put_intent(intent))?,
            Self::U8 => writeln!(stream, "{}<type name=\"u8\"/>", put_intent(intent))?,
            Self::U16 => writeln!(stream, "{}<type name=\"u16\"/>", put_intent(intent))?,
            Self::U32 => writeln!(stream, "{}<type name=\"u32\"/>", put_intent(intent))?,
            Self::U64 => writeln!(stream, "{}<type name=\"u64\"/>", put_intent(intent))?,
            Self::U128 => writeln!(stream, "{}<type name=\"u128\"/>", put_intent(intent))?,
            Self::F32 => writeln!(stream, "{}<type name=\"f32\"/>", put_intent(intent))?,
            Self::F64 => writeln!(stream, "{}<type name=\"f64\"/>", put_intent(intent))?,
            Self::Str => writeln!(stream, "{}<type name=\"str\"/>", put_intent(intent))?,
        }

        Ok(())
    }

    fn codegen(&self, stream: &mut Stream) -> std::io::Result<()> {
        write!(stream, " {} ", self.get_c_type())
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ref { is_mut, ref_to } => {
                write!(f, "&")?;

                if *is_mut {
                    write!(f, "mut")?;
                }

                write!(f, "{}", ref_to)
            }
            Self::Ptr { is_mut, ptr_to } => {
                write!(f, "*")?;

                if *is_mut {
                    write!(f, "mut")?;
                }

                write!(f, "{}", ptr_to)
            }
            Self::Template {
                identifier,
                arguments,
            } => {
                write!(f, "{}<", identifier)?;
                for i in arguments.iter() {
                    write!(f, "{}", i)?;
                }
                write!(f, ">")
            }
            Self::Tuple { components } => {
                write!(f, "( ")?;

                for i in components.iter() {
                    write!(f, "{} ", i)?;
                }

                write!(f, ")")
            }
            Self::Array { value_type, .. } => write!(f, "[{}]", value_type),
            Self::Custom(s) => write!(f, "{}", s),
            Self::Bool => write!(f, "bool"),
            Self::I8 => write!(f, "i8"),
            Self::I16 => write!(f, "i16"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::I128 => write!(f, "i128"),
            Self::U8 => write!(f, "u8"),
            Self::U16 => write!(f, "u16"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
            Self::U128 => write!(f, "u128"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::Str => write!(f, "str"),
        }
    }
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::new()
    }
}

impl IAst for Alias {
    fn get_type(&self, _analyzer: &mut crate::analyzer::Analyzer) -> self::Type {
        self.value.clone()
    }

    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            analyzer.error(&format!(
                "Identifier \"{}\" defined multiple times",
                &self.identifier
            ));
            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
        }

        analyzer.add_symbol(&self.identifier, analyzer.create_symbol(SymbolData::Type));

        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<alias name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        self.value.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</alias>", put_intent(intent))?;

        Ok(())
    }

    fn codegen(&self, stream: &mut Stream) -> std::io::Result<()> {
        write!(stream, "typedef {} ", self.value.get_c_type())?;

        self.identifier.codegen(stream)?;

        writeln!(stream, ";\n")
    }
}
