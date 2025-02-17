use super::{MetaInfo, TypeSpec};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_parser::Parser;
use tanitc_ty::Type;

impl TypeSpec {
    pub fn parse(parser: &mut Parser) -> Result<Self, Message> {
        let location = parser.peek_token().location;
        let (ty, info) = Self::parse_type(parser)?;

        Ok(Self { location, info, ty })
    }

    fn parse_type(parser: &mut Parser) -> Result<(Type, MetaInfo), Message> {
        let mut info = MetaInfo::default();
        let next = parser.peek_token();

        if parser.peek_token().lexem == Lexem::Ampersand {
            info.is_mut = false;
            parser.get_token();

            if matches!(parser.peek_token().lexem, Lexem::KwMut) {
                info.is_mut = true;
                parser.get_token();
            }

            let (ref_to, _) = Self::parse_type(parser)?;

            return Ok((Type::Ref(Box::new(ref_to)), info));
        }

        if next.lexem == Lexem::Star {
            info.is_mut = false;
            parser.get_token();

            if matches!(parser.peek_token().lexem, Lexem::KwMut) {
                info.is_mut = true;
                parser.get_token();
            }

            let (ptr_to, _) = Self::parse_type(parser)?;

            return Ok((Type::Ptr(Box::new(ptr_to)), info));
        }

        if next.lexem == Lexem::LParen {
            return Self::parse_tuple_def(parser);
        }

        if next.lexem == Lexem::Lsb {
            return Self::parse_array_def(parser);
        }

        let identifier = parser.consume_identifier()?;
        let id_str: String = identifier.into();

        match &id_str[..] {
            "!" => return Ok((Type::Never, info)),
            "bool" => return Ok((Type::Bool, info)),
            "i8" => return Ok((Type::I8, info)),
            "i16" => return Ok((Type::I16, info)),
            "i32" => return Ok((Type::I32, info)),
            "i64" => return Ok((Type::I64, info)),
            "i128" => return Ok((Type::I128, info)),
            "u8" => return Ok((Type::U8, info)),
            "u16" => return Ok((Type::U16, info)),
            "u32" => return Ok((Type::U32, info)),
            "u64" => return Ok((Type::U64, info)),
            "u128" => return Ok((Type::U128, info)),
            "f32" => return Ok((Type::F32, info)),
            "f64" => return Ok((Type::F64, info)),
            "str" => return Ok((Type::Str, info)),
            _ => {}
        }

        if parser.peek_singular().lexem == Lexem::Lt {
            return Ok((
                Type::Template {
                    identifier,
                    generics: Self::parse_template_generics(parser)?,
                },
                info,
            ));
        }

        Ok((Type::Custom(id_str), info))
    }

    pub fn parse_tuple_def(parser: &mut Parser) -> Result<(Type, MetaInfo), Message> {
        parser.consume_token(Lexem::LParen)?;

        let mut children = Vec::<Type>::new();
        loop {
            if parser.peek_token().lexem == Lexem::RParen {
                break;
            }

            let (child, _) = Self::parse_type(parser)?;
            children.push(child);

            if parser.peek_token().lexem == Lexem::Comma {
                parser.get_token();
                continue;
            }
        }

        parser.consume_token(Lexem::RParen)?;

        Ok((Type::Tuple(children), MetaInfo::default()))
    }

    pub fn parse_array_def(parser: &mut Parser) -> Result<(Type, MetaInfo), Message> {
        parser.consume_token(Lexem::Lsb)?;

        let (value_type, _) = Self::parse_type(parser)?;

        if parser.peek_token().lexem == Lexem::Colon {
            parser.get_token();

            // size = Some(Box::new(Expression::parse(parser)?));
        }

        parser.consume_token(Lexem::Rsb)?;

        Ok((
            Type::Array {
                size: None,
                value_type: Box::new(value_type),
            },
            MetaInfo::default(),
        ))
    }

    fn parse_template_generics(parser: &mut Parser) -> Result<Vec<Type>, Message> {
        parser.consume_token(Lexem::Lt)?;

        let mut children = Vec::<Type>::new();
        loop {
            let (child, _) = Self::parse_type(parser)?;
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
}
