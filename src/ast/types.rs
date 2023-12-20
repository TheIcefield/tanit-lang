use crate::ast::{Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::fmt::Debug;
use std::io::Write;

use super::expressions::parse_expression;
use super::GetType;

#[derive(Clone)]
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
    Byte,
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
    pub fn parse(parser: &mut Parser) -> Option<Type> {
        let next = parser.peek_token();

        if parser.peek_token().lexem == TokenType::Ampersand {
            let mut is_mut = false;
            parser.get_token();

            if matches!(parser.peek_token().lexem, TokenType::KwMut) {
                is_mut = true;
                parser.get_token();
            }

            return Some(Type::Ref {
                is_mut,
                ref_to: Box::new(Self::parse(parser)?),
            });
        }

        if next.lexem == TokenType::Star {
            let mut is_mut = false;
            parser.get_token();

            if matches!(parser.peek_token().lexem, TokenType::KwMut) {
                is_mut = true;
                parser.get_token();
            }

            return Some(Type::Ptr {
                is_mut,
                ptr_to: Box::new(Self::parse(parser)?),
            });
        }

        if next.lexem == TokenType::LParen {
            return Self::parse_tuple_def(parser);
        }

        if next.lexem == TokenType::Lsb {
            return Self::parse_array_def(parser);
        }

        let identifier = parser.consume_identifier()?;

        match &identifier[..] {
            "bool" => return Some(Type::Bool),
            "byte" => return Some(Type::Byte),
            "i8" => return Some(Type::I8),
            "i16" => return Some(Type::I16),
            "i32" => return Some(Type::I32),
            "i64" => return Some(Type::I64),
            "i128" => return Some(Type::I128),
            "u8" => return Some(Type::U8),
            "u16" => return Some(Type::U16),
            "u32" => return Some(Type::U32),
            "u64" => return Some(Type::U64),
            "u128" => return Some(Type::U128),
            "f32" => return Some(Type::F32),
            "f64" => return Some(Type::F64),
            "str" => return Some(Type::Str),
            _ => {}
        }

        if parser.peek_singular().lexem == TokenType::Lt {
            let arguments = Self::parse_template_args(parser)?;

            return Some(Type::Template {
                identifier,
                arguments,
            });
        }

        Some(Type::Custom(identifier))
    }

    pub fn parse_tuple_def(parser: &mut Parser) -> Option<Type> {
        parser.consume_token(TokenType::LParen)?;

        let mut children = Vec::<Type>::new();
        loop {
            if parser.peek_token().lexem == TokenType::RParen {
                break;
            }

            let child = Self::parse(parser)?;
            children.push(child);

            if parser.peek_token().lexem == TokenType::Comma {
                parser.get_token();
                continue;
            }
        }

        parser.consume_token(TokenType::RParen)?;

        Some(Type::Tuple {
            components: children,
        })
    }

    pub fn parse_array_def(parser: &mut Parser) -> Option<Type> {
        parser.consume_token(TokenType::Lsb)?;

        let mut size: Option<Box<Ast>> = None;

        let value_type = Box::new(Self::parse(parser)?);

        if parser.peek_token().lexem == TokenType::Colon {
            parser.get_token();

            size = Some(Box::new(parse_expression(parser)?));
        }

        parser.consume_token(TokenType::Rsb)?;

        Some(Type::Array { size, value_type })
    }

    pub fn parse_template_args(parser: &mut Parser) -> Option<Vec<Type>> {
        parser.consume_token(TokenType::Lt)?;

        let mut children = Vec::<Type>::new();
        loop {
            let child = Self::parse(parser)?;
            children.push(child);

            let next = parser.peek_singular();
            if next.lexem == TokenType::Gt {
                break;
            } else {
                parser.consume_token(TokenType::Comma)?;
            }
        }

        parser.get_singular();

        Some(children)
    }
}

#[derive(Clone)]
pub struct Alias {
    pub identifier: Id,
    pub value: Type,
}

impl Alias {
    pub fn parse_def(parser: &mut Parser) -> Option<Ast> {
        parser.consume_token(TokenType::KwAlias)?;

        let identifier = parser.consume_identifier()?;

        parser.consume_token(TokenType::Assign)?;

        let value = Type::parse(parser)?;

        Some(Ast::AliasDef {
            node: Alias { identifier, value },
        })
    }
}

impl IAst for Type {
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
                    "{}<type identifier=\"{}\">",
                    put_intent(intent),
                    identifier
                )?;

                for arg in arguments.iter() {
                    arg.traverse(stream, intent + 1)?;
                }

                writeln!(stream, "{}</type>", put_intent(intent))?;
            }

            Self::Custom(id) => {
                writeln!(
                    stream,
                    "{}<type identifier=\"{}\"/>",
                    put_intent(intent),
                    id
                )?;
            }

            Self::Bool => writeln!(stream, "{}<type identifier=\"bool\"/>", put_intent(intent))?,
            Self::Byte => writeln!(stream, "{}<type identifier=\"byte\"/>", put_intent(intent))?,
            Self::I8 => writeln!(stream, "{}<type identifier=\"i8\"/>", put_intent(intent))?,
            Self::I16 => writeln!(stream, "{}<type identifier=\"i16\"/>", put_intent(intent))?,
            Self::I32 => writeln!(stream, "{}<type identifier=\"i32\"/>", put_intent(intent))?,
            Self::I64 => writeln!(stream, "{}<type identifier=\"i64\"/>", put_intent(intent))?,
            Self::I128 => writeln!(stream, "{}<type identifier=\"i128\"/>", put_intent(intent))?,
            Self::U8 => writeln!(stream, "{}<type identifier=\"u8\"/>", put_intent(intent))?,
            Self::U16 => writeln!(stream, "{}<type identifier=\"u16\"/>", put_intent(intent))?,
            Self::U32 => writeln!(stream, "{}<type identifier=\"u32\"/>", put_intent(intent))?,
            Self::U64 => writeln!(stream, "{}<type identifier=\"u64\"/>", put_intent(intent))?,
            Self::U128 => writeln!(stream, "{}<type identifier=\"u128\"/>", put_intent(intent))?,
            Self::F32 => writeln!(stream, "{}<type identifier=\"f32\"/>", put_intent(intent))?,
            Self::F64 => writeln!(stream, "{}<type identifier=\"f64\"/>", put_intent(intent))?,
            Self::Str => writeln!(stream, "{}<type identifier=\"str\"/>", put_intent(intent))?,
        }

        Ok(())
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ref { is_mut, ref_to } => {
                write!(f, "&")?;

                if *is_mut {
                    write!(f, "mut")?;
                }

                write!(f, "{:?}", ref_to)
            }
            Self::Ptr { is_mut, ptr_to } => {
                write!(f, "*")?;

                if *is_mut {
                    write!(f, "mut")?;
                }

                write!(f, "{:?}", ptr_to)
            }
            Self::Template {
                identifier,
                arguments,
            } => {
                write!(f, "{}<", identifier)?;
                for i in arguments.iter() {
                    write!(f, "{:?}", i)?;
                }
                write!(f, ">")
            }
            Self::Tuple { components } => {
                write!(f, "(")?;

                for i in components.iter() {
                    write!(f, "{:?}", i)?;
                }

                write!(f, ")")
            }
            Self::Array { value_type, .. } => write!(f, "[{:?}]", value_type),
            Self::Custom(s) => write!(f, "{}", s),
            Self::Bool => writeln!(f, "bool"),
            Self::Byte => writeln!(f, "byte"),
            Self::I8 => writeln!(f, "i8"),
            Self::I16 => writeln!(f, "i16"),
            Self::I32 => writeln!(f, "i32"),
            Self::I64 => writeln!(f, "i64"),
            Self::I128 => writeln!(f, "i128"),
            Self::U8 => writeln!(f, "u8"),
            Self::U16 => writeln!(f, "u16"),
            Self::U32 => writeln!(f, "u32"),
            Self::U64 => writeln!(f, "u64"),
            Self::U128 => writeln!(f, "u128"),
            Self::F32 => writeln!(f, "f32"),
            Self::F64 => writeln!(f, "f64"),
            Self::Str => writeln!(f, "str"),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        !self.ne(other)
    }
}

impl IAst for Alias {
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
}

impl GetType for Alias {
    fn get_type(&self) -> Option<Type> {
        Some(self.value.clone())
    }
}
