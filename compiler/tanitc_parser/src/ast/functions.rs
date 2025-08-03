use tanitc_ast::ast::{
    functions::{FunctionDef, FunctionParam},
    types::{ParsedTypeInfo, TypeSpec},
    variables::{VariableAttributes, VariableDef},
    Ast,
};
use tanitc_attributes::Mutability;
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

    fn parse_func_self_val_param(
        &mut self,
        mutability: Mutability,
    ) -> Result<FunctionParam, Message> {
        self.consume_token(Lexem::KwSelfO)?;

        Ok(FunctionParam::SelfVal(mutability))
    }

    fn parse_func_self_ref_param(
        &mut self,
        mutability: Mutability,
    ) -> Result<FunctionParam, Message> {
        let location = self.consume_token(Lexem::Ampersand)?.location;

        if mutability.is_mutable() {
            return Err(Message::new(
                location,
                "\"Mut\" must be followed named binding",
            ));
        }

        let mutability = if self.peek_token().lexem == Lexem::KwMut {
            self.get_token(); // mut
            Mutability::Mutable
        } else {
            Mutability::default()
        };

        self.consume_token(Lexem::KwSelfO)?;

        Ok(FunctionParam::SelfRef(mutability))
    }

    fn parse_func_common_param(
        &mut self,
        mutability: Mutability,
    ) -> Result<FunctionParam, Message> {
        let location = self.peek_token().location;
        let identifier = self.consume_identifier()?;

        self.consume_token(Lexem::Colon)?;

        let var_type = self.parse_type_spec()?;

        Ok(FunctionParam::Common(VariableDef {
            location,
            attributes: VariableAttributes::default(),
            identifier,
            var_type,
            is_global: false,
            mutability,
        }))
    }

    fn parse_func_header_param(&mut self) -> Result<FunctionParam, Message> {
        let next = self.peek_token();

        let mut mutability = Mutability::default();
        if next.lexem == Lexem::KwMut {
            self.get_token();
            mutability = Mutability::Mutable;
        }

        let next = self.peek_token();
        match next.lexem {
            Lexem::KwSelfO => self.parse_func_self_val_param(mutability),
            Lexem::Ampersand => self.parse_func_self_ref_param(mutability),
            Lexem::Identifier(_) => self.parse_func_common_param(mutability),
            _ => Err(Message::unexpected_token(next, &[])),
        }
    }

    fn parse_func_header_params(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        self.consume_token(Lexem::LParen)?;

        loop {
            let next = self.peek_token();
            if next.lexem == Lexem::RParen {
                self.get_token();
                break;
            }

            match self.parse_func_header_param() {
                Ok(param) => func_def.parameters.push(param),
                Err(err) => {
                    self.error(Message::in_func_def(func_def.identifier, err));
                }
            }

            let next = self.peek_token();
            if next.lexem == Lexem::Comma {
                self.get_token();
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
                let Ast::Block(block) = self.parse_local_block()? else {
                    unreachable!();
                };

                func_def.body = Some(Box::new(block));
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
