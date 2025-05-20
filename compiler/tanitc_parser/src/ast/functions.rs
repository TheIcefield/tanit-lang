use tanitc_ast::{Ast, FunctionDef, ParsedTypeInfo, TypeSpec, VariableDef};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::Parser;

impl Parser {
    pub fn parse_func_def(&mut self) -> Result<Ast, Message> {
        let mut node = FunctionDef::default();

        self.parse_func_header(&mut node)?;
        self.parse_func_body(&mut node)?;

        Ok(Ast::from(node))
    }
}

// Private
impl Parser {
    fn parse_func_header(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        func_def.location = self.consume_token(Lexem::KwFunc)?.location;
        func_def.identifier = self.consume_identifier()?;

        self.parse_func_header_params(func_def)?;
        self.parse_func_return_type(func_def)?;

        Ok(())
    }

    fn parse_func_header_params(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        self.consume_token(Lexem::LParen)?;

        loop {
            let next = self.peek_token();

            let mut is_mutable = false;
            if next.lexem == Lexem::KwMut {
                self.get_token();
                is_mutable = true;
            }

            let next = self.peek_token();

            if next.is_identifier() {
                let mut param = self.parse_func_param()?;
                param.is_mutable = is_mutable;

                func_def.parameters.push(Ast::VariableDef(param));

                let next = self.peek_token();
                if next.lexem == Lexem::Comma {
                    self.get_token();
                } else if next.lexem == Lexem::RParen {
                    continue;
                } else {
                    return Err(Message::unexpected_token(next, &[]));
                }
            } else if next.lexem == Lexem::RParen {
                self.get_token();
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        Ok(())
    }

    fn parse_func_body(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        self.parse_func_body_internal(func_def)?;

        self.set_ignore_nl_option(old_opt);

        Ok(())
    }

    fn parse_func_body_internal(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        let next = self.peek_token();

        match next.lexem {
            Lexem::EndOfLine => {}

            Lexem::Lcb => {
                func_def.body = Some(Box::new(self.parse_local_block()?));
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::Lcb, Lexem::EndOfLine],
                ));
            }
        }

        Ok(())
    }

    fn parse_func_param(&mut self) -> Result<VariableDef, Message> {
        let location = self.peek_token().location;
        let identifier = self.consume_identifier()?;

        self.consume_token(Lexem::Colon)?;

        let var_type = self.parse_type_spec()?;

        Ok(VariableDef {
            location,
            attributes: tanitc_ast::attributes::VariableAttributes::default(),
            identifier,
            var_type,
            is_global: false,
            is_mutable: false,
        })
    }

    fn parse_func_return_type(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        let next = self.peek_token();
        func_def.return_type = if Lexem::Colon == next.lexem {
            self.get_token();
            self.parse_type_spec()?
        } else {
            TypeSpec {
                location: next.location,
                info: ParsedTypeInfo::default(),
                ty: Type::unit(),
            }
        };

        self.set_ignore_nl_option(old_opt);

        Ok(())
    }
}

#[test]
fn parse_func_def_test() {
    const SRC_TEXT: &str = "func hello(a: i32): i32 {\
                          \n    return a\
                          \n}";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();
    let ast = parser.parse_func_def().unwrap();

    let Ast::FuncDef(func_node) = &ast else {
        panic!("Expected FuncDef, actually: {}", ast.name());
    };

    assert_eq!(func_node.identifier.to_string(), "hello");
    assert!(func_node.body.is_some());
    assert_eq!(func_node.return_type.get_type(), Type::I32);
    assert_eq!(func_node.parameters.len(), 1);
}
