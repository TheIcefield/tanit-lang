use tanitc_ast::ast::types::{ParsedTypeInfo, TypeSpec};
use tanitc_attributes::Mutability;
use tanitc_ident::Name;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::{ArraySize, Type};

use crate::Parser;

impl Parser {
    pub fn parse_type_spec(&mut self) -> Result<TypeSpec, Message> {
        let location = self.peek_token().location;
        let (ty, info) = self.parse_type()?;

        Ok(TypeSpec { location, info, ty })
    }

    fn parse_reference_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::Ampersand)?;

        let mut info = ParsedTypeInfo::default();

        if matches!(self.peek_token().lexem, Lexem::KwMut) {
            info.mutability = Mutability::Mutable;
            self.get_token();
        }

        let (ref_to, _) = self.parse_type()?;

        Ok((
            Type::Ref {
                ref_to: Box::new(ref_to),
                mutability: info.mutability,
            },
            info,
        ))
    }

    fn parse_pointer_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::Star)?;

        let mut info = ParsedTypeInfo::default();

        if matches!(self.peek_token().lexem, Lexem::KwMut) {
            info.mutability = Mutability::Mutable;
            self.get_token();
        }

        let (ptr_to, _) = self.parse_type()?;

        Ok((Type::Ptr(Box::new(ptr_to)), info))
    }

    fn parse_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        let next = self.peek_token();

        // Parse reference: &mut i32
        if self.peek_token().lexem == Lexem::Ampersand {
            return self.parse_reference_type();
        }

        // Parse pointer: *mut f32
        if next.lexem == Lexem::Star {
            return self.parse_pointer_type();
        }

        // Parse tuple: (i32, f32)
        if next.lexem == Lexem::LParen {
            return self.parse_tuple_def();
        }

        // Parse array: [f32: 4]
        if next.lexem == Lexem::Lsb {
            return self.parse_array_def();
        }

        let identifier = self.consume_identifier()?;
        let id_str: String = identifier.into();

        let info = ParsedTypeInfo::default();
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

        if self.peek_singular().lexem == Lexem::Lt {
            return Ok((
                Type::Template {
                    identifier,
                    generics: self.parse_template_generics()?,
                },
                info,
            ));
        }

        Ok((Type::Custom(Name::from(id_str.to_string())), info))
    }

    pub fn parse_tuple_def(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::LParen)?;

        let mut children = Vec::<Type>::new();
        loop {
            if self.peek_token().lexem == Lexem::RParen {
                break;
            }

            let (child, _) = self.parse_type()?;
            children.push(child);

            if self.peek_token().lexem == Lexem::Comma {
                self.get_token();
                continue;
            }
        }

        self.consume_token(Lexem::RParen)?;

        Ok((Type::Tuple(children), ParsedTypeInfo::default()))
    }

    fn parse_array_def(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::Lsb)?;

        let (value_type, _) = self.parse_type()?;
        let mut size = ArraySize::Unknown;

        if self.peek_token().lexem == Lexem::Colon {
            self.get_token();

            size = ArraySize::Fixed(self.consume_integer()?.lexem.get_int().unwrap());
        }

        self.consume_token(Lexem::Rsb)?;

        Ok((
            Type::Array {
                size,
                value_type: Box::new(value_type),
            },
            ParsedTypeInfo::default(),
        ))
    }

    fn parse_template_generics(&mut self) -> Result<Vec<Type>, Message> {
        self.consume_token(Lexem::Lt)?;

        let mut children = Vec::<Type>::new();
        loop {
            let (child, _) = self.parse_type()?;
            children.push(child);

            let next = self.peek_singular();
            if next.lexem == Lexem::Gt {
                break;
            } else {
                self.consume_token(Lexem::Comma)?;
            }
        }

        self.get_singular();

        Ok(children)
    }
}
