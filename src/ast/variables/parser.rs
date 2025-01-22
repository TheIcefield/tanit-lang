use super::VariableDef;
use crate::ast::{
    expressions::{Expression, ExpressionType},
    identifiers::Identifier,
    types::Type,
    Ast,
};
use crate::messages::Message;
use crate::parser::{token::Lexem, Parser};

impl VariableDef {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let next = parser.peek_token();
        let location = next.location;

        let is_global = match next.lexem {
            Lexem::KwLet => {
                parser.get_token();
                false
            }

            Lexem::KwStatic => {
                parser.get_token();
                true
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::KwLet, Lexem::KwStatic],
                ));
            }
        };

        let next = parser.peek_token();
        let is_mutable = match next.lexem {
            Lexem::KwMut => {
                parser.get_token();
                true
            }

            Lexem::KwConst => {
                parser.get_token();
                false
            }

            _ => false,
        };

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        let next = parser.peek_token();

        let mut var_type: Option<Type> = None;
        let mut rvalue: Option<Ast> = None;

        if Lexem::Colon == next.lexem {
            parser.consume_token(Lexem::Colon)?;

            var_type = Some(Type::parse(parser)?);
        }

        let next = parser.peek_token();

        if Lexem::Assign == next.lexem {
            parser.get_token();

            rvalue = Some(Expression::parse(parser)?);
        }

        if var_type.is_none() && rvalue.is_none() {
            return Err(Message::new(
                location,
                &format!(
                    "Variable {} defined without type. Need to specify type or use with rvalue",
                    identifier
                ),
            ));
        }

        if var_type.is_none() && is_global {
            return Err(Message::new(
                location,
                &format!(
                    "Variable {} defined without type, but marked as static. Need to specify type",
                    identifier
                ),
            ));
        }

        if var_type.is_none() && rvalue.is_some() {
            var_type = Some(Type::Auto);
        }

        let var_node = Ast::VariableDef {
            node: Self {
                location,
                identifier,
                var_type: var_type.unwrap_or(Type::Auto),
                is_global,
                is_mutable,
            },
        };

        if let Some(rhs) = rvalue {
            return Ok(Ast::Expression {
                node: Expression {
                    location,
                    expr: ExpressionType::Binary {
                        operation: Lexem::Assign,
                        lhs: Box::new(var_node),
                        rhs: Box::new(rhs),
                    },
                },
            });
        }

        Ok(var_node)
    }
}
