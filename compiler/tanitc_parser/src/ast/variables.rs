use tanitc_ast::{
    attributes::VariableAttributes, Ast, Expression, ExpressionKind, ParsedTypeInfo, TypeSpec,
    VariableDef,
};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::Parser;

impl Parser {
    pub fn parse_variable_def(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token();
        let location = next.location;

        let is_global = match next.lexem {
            Lexem::KwVar => {
                self.get_token();
                false
            }

            Lexem::KwStatic => {
                self.get_token();
                true
            }

            _ => {
                return Err(Message::unexpected_token(
                    next,
                    &[Lexem::KwVar, Lexem::KwStatic],
                ));
            }
        };

        let next = self.peek_token();
        let is_mutable = match next.lexem {
            Lexem::KwMut => {
                self.get_token();
                true
            }

            Lexem::KwConst => {
                self.get_token();
                false
            }

            _ => false,
        };

        let identifier = self.consume_identifier()?;

        let next = self.peek_token();

        let mut var_type: Option<TypeSpec> = None;
        let mut rvalue: Option<Ast> = None;

        if Lexem::Colon == next.lexem {
            self.consume_token(Lexem::Colon)?;

            var_type = Some(self.parse_type_spec()?);
        }

        let next = self.peek_token();

        if Lexem::Assign == next.lexem {
            self.get_token();

            rvalue = Some(self.parse_expression()?);
        }

        if var_type.is_none() && rvalue.is_none() {
            return Err(Message::from_string(
                location,
                format!(
                    "Variable \"{identifier}\" defined without type. Need to specify type or use with rvalue"
                ),
            ));
        }

        if var_type.is_none() && is_global {
            return Err(Message::from_string(
                location,
                format!(
                    "Variable {identifier} defined without type, but marked as static. Need to specify type"
                ),
            ));
        }

        let var_node = Ast::from(VariableDef {
            location,
            attributes: VariableAttributes::default(),
            identifier,
            var_type: var_type.unwrap_or(TypeSpec {
                location,
                info: ParsedTypeInfo::default(),
                ty: Type::Auto,
            }),
            is_global,
            is_mutable,
        });

        if let Some(rhs) = rvalue {
            return Ok(Ast::from(Expression {
                location,
                kind: ExpressionKind::new_binary(Lexem::Assign, Box::new(var_node), Box::new(rhs))?,
            }));
        }

        Ok(var_node)
    }
}
