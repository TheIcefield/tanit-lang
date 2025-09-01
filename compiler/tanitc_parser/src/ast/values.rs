use tanitc_ast::ast::{
    values::{CallArg, CallArgKind, Value, ValueKind},
    Ast,
};
use tanitc_ident::Name;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    // func_name(1, 2 ,3)
    pub fn parse_call_params(&mut self) -> Result<Vec<CallArg>, Message> {
        let _ = self.consume_token(Lexem::LParen)?.location;

        let mut args = Vec::<CallArg>::new();

        let mut i = 0;
        loop {
            let next = self.peek_token();

            if next.lexem == Lexem::RParen {
                break;
            }

            let expr = self.parse_expression()?;

            let arg_location = expr.location();

            let arg_id = if let Ast::Value(Value {
                kind: ValueKind::Identifier(id),
                ..
            }) = &expr
            {
                if self.peek_token().lexem == Lexem::Colon {
                    self.consume_token(Lexem::Colon)?;
                    Some(*id)
                } else {
                    None
                }
            } else {
                None
            };

            let arg_kind = if let Some(id) = &arg_id {
                CallArgKind::Notified(*id, Box::new(self.parse_expression()?))
            } else {
                CallArgKind::Positional(i, Box::new(expr))
            };

            args.push(CallArg {
                location: arg_location,
                identifier: arg_id,
                kind: arg_kind,
            });

            i += 1;

            let next = self.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                self.get_token();
                continue;
            } else if next.lexem == Lexem::RParen {
                // end parsing if ')'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        self.consume_token(Lexem::RParen)?;

        Ok(args)
    }

    // [1, 2, 3]
    pub fn parse_array_value(&mut self) -> Result<Ast, Message> {
        let location = self.consume_token(Lexem::Lsb)?.location;

        let mut components = Vec::<Ast>::new();

        loop {
            let next = self.peek_token();

            if next.lexem == Lexem::Rsb {
                break;
            }
            components.push(self.parse_expression()?);

            let next = self.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                self.get_token();
                continue;
            } else if next.lexem == Lexem::Rsb {
                // end parsing if ']'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        self.consume_token(Lexem::Rsb)?;

        Ok(Ast::from(Value {
            location,
            kind: ValueKind::Array { components },
        }))
    }

    // StructName { field_1: i32, field2: f32 }
    pub fn parse_struct_value(&mut self) -> Result<Vec<(Name, Ast)>, Message> {
        self.consume_token(Lexem::Lcb)?;

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(true);

        let components = self.parse_struct_value_internal()?;

        self.set_ignore_nl_option(old_opt);
        self.consume_token(Lexem::Rcb)?;

        Ok(components)
    }

    fn parse_struct_value_internal(&mut self) -> Result<Vec<(Name, Ast)>, Message> {
        let mut components = Vec::<(Name, Ast)>::new();

        loop {
            let next = self.peek_token();

            if next.lexem == Lexem::Rcb {
                break;
            }

            let identifier = self.consume_identifier()?;

            self.consume_token(Lexem::Colon)?;

            components.push((
                Name {
                    id: identifier,
                    ..Default::default()
                },
                self.parse_expression()?,
            ));

            let next = self.peek_token();
            if next.lexem == Lexem::Comma || next.lexem == Lexem::EndOfLine {
                // continue parsing if ','
                self.get_token();
                continue;
            } else if next.lexem == Lexem::Rcb {
                // end parsing if '}'
                break;
            } else {
                return Err(Message::unexpected_token(next, &[]));
            }
        }

        Ok(components)
    }
}
