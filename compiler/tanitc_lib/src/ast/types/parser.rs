use super::Type;
use crate::ast::{expressions::Expression, identifiers::Identifier, Ast};
use crate::messages::Message;
use crate::parser::Parser;

use tanitc_lexer::token::Lexem;

impl Type {
    pub fn parse(parser: &mut Parser) -> Result<Self, Message> {
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

        let identifier = parser.consume_identifier()?;
        let id_str = identifier.lexem.get_string();

        match &id_str[..] {
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
                identifier: Identifier::from_token(&identifier)?,
                arguments,
            });
        }

        Ok(Type::Custom(id_str))
    }

    pub fn parse_tuple_def(parser: &mut Parser) -> Result<Self, Message> {
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

    pub fn parse_array_def(parser: &mut Parser) -> Result<Self, Message> {
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

    pub fn parse_template_args(parser: &mut Parser) -> Result<Vec<Self>, Message> {
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

    pub fn get_c_type(&self) -> String {
        match self {
            Self::Auto => unreachable!("automatic type is not eliminated"),
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
            Self::Custom(id) => id,
            Self::Tuple { components } => {
                if components.is_empty() {
                    "void"
                } else {
                    unimplemented!()
                }
            }
            _ => unimplemented!(),
        }
        .to_string()
    }
}
