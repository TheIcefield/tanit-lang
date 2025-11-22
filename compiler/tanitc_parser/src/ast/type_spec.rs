use tanitc_ast::ast::types::{ParsedTypeInfo, TypeSpec};
use tanitc_attributes::Mutability;
use tanitc_ident::Name;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::{ArraySize, FuncType, RefType, Type};

use crate::Parser;

impl Parser {
    pub fn parse_type_spec(&mut self) -> Result<TypeSpec, Message> {
        let location = self
            .peek_token()
            .ok_or(Message::reached_eof())?
            .location_ref()
            .clone();

        let (ty, info) = self.parse_type()?;

        Ok(TypeSpec { location, info, ty })
    }

    fn parse_func_type_params(&mut self) -> Result<Vec<Type>, Message> {
        self.consume_token(Lexem::LParen)?;

        let mut params = Vec::<Type>::new();

        loop {
            let Some(tkn) = self.peek_token() else {
                break;
            };

            if *tkn.lexem_ref() == Lexem::RParen {
                self.get_token();
                break;
            }

            let ty = self.parse_type()?;

            params.push(ty.0);
        }

        Ok(params)
    }

    fn parse_func_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::KwFunc)?;

        let parameters = self.parse_func_type_params()?;

        let return_type = if self.is_next(Lexem::Colon) {
            self.get_token();
            Box::new(self.parse_type()?.0)
        } else {
            Box::new(Type::unit())
        };

        Ok((
            Type::Func(FuncType {
                parameters,
                return_type,
            }),
            ParsedTypeInfo::default(),
        ))
    }

    fn parse_reference_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::Ampersand)?;

        let mut info = ParsedTypeInfo::default();

        if self.is_next(Lexem::KwMut) {
            info.mutability = Mutability::Mutable;
            self.get_token();
        }

        let (ref_to, _) = self.parse_type()?;

        Ok((
            Type::Ref(RefType {
                ref_to: Box::new(ref_to),
                mutability: info.mutability,
            }),
            info,
        ))
    }

    fn parse_pointer_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::Star)?;

        let mut info = ParsedTypeInfo::default();

        if self.is_next(Lexem::KwMut) {
            info.mutability = Mutability::Mutable;
            self.get_token();
        }

        let (ptr_to, _) = self.parse_type()?;

        Ok((Type::Ptr(Box::new(ptr_to)), info))
    }

    fn parse_named_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
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

        if Lexem::Lt == *self.peek_token().ok_or(Message::reached_eof())?.lexem_ref() {
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

    fn parse_type(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        match next.lexem_ref() {
            Lexem::KwFunc => self.parse_func_type(),
            Lexem::Ampersand => self.parse_reference_type(),
            Lexem::Star => self.parse_pointer_type(),
            Lexem::LParen => self.parse_tuple_def(),
            Lexem::Lsb => self.parse_array_def(),
            Lexem::Identifier(_) => self.parse_named_type(),
            _ => Err(Message::unexpected_token(&next, &[])),
        }
    }

    pub fn parse_tuple_def(&mut self) -> Result<(Type, ParsedTypeInfo), Message> {
        self.consume_token(Lexem::LParen)?;

        let mut children = Vec::<Type>::new();
        loop {
            if self.is_next(Lexem::RParen) {
                break;
            }

            let (child, _) = self.parse_type()?;
            children.push(child);

            if self.is_next(Lexem::Comma) {
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

        if self.is_next(Lexem::Colon) {
            self.get_token();

            size = ArraySize::Fixed(self.consume_integer()?.lexem_ref().get_int().unwrap());
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

            let next = self.peek_token().ok_or(Message::reached_eof())?;
            if *next.lexem_ref() == Lexem::Gt {
                break;
            } else {
                self.consume_token(Lexem::Comma)?;
            }
        }

        self.get_token();

        Ok(children)
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ty::Type;

    use crate::Parser;

    #[test]
    fn parse_empty_func_type_test() {
        const SRC_TEXT: &str = "func()";

        let mut parser = Parser::from_text(SRC_TEXT);
        let ty = parser.parse_type().unwrap();

        let Type::Func(func_type) = &ty.0 else {
            panic!("Expected Type::Func");
        };

        assert!(func_type.parameters.is_empty());
        assert_eq!(*func_type.return_type, Type::unit());
    }

    #[test]
    fn parse_empty_func_type_with_ret_test() {
        const SRC_TEXT: &str = "func():i32";

        let mut parser = Parser::from_text(SRC_TEXT);
        let ty = parser.parse_type().unwrap();

        let Type::Func(func_type) = &ty.0 else {
            panic!("Expected Type::Func");
        };

        assert!(func_type.parameters.is_empty());
        assert_eq!(*func_type.return_type, Type::I32);
    }

    #[test]
    fn parse_func_type_test() {
        const SRC_TEXT: &str = "func(i32)";

        let mut parser = Parser::from_text(SRC_TEXT);
        let ty = parser.parse_type().unwrap();

        let Type::Func(func_type) = &ty.0 else {
            panic!("Expected Type::Func");
        };

        assert_eq!(func_type.parameters.len(), 1);
        assert_eq!(func_type.parameters[0], Type::I32);
        assert_eq!(*func_type.return_type, Type::unit());
    }

    #[test]
    fn parse_func_type_with_ret_test() {
        const SRC_TEXT: &str = "func(i32):i32";

        let mut parser = Parser::from_text(SRC_TEXT);
        let ty = parser.parse_type().unwrap();

        let Type::Func(func_type) = &ty.0 else {
            panic!("Expected Type::Func");
        };

        assert_eq!(func_type.parameters.len(), 1);
        assert_eq!(func_type.parameters[0], Type::I32);
        assert_eq!(*func_type.return_type, Type::I32);
    }
}
