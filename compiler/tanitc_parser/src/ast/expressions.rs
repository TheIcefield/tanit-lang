use tanitc_ast::ast::{
    expressions::{BinaryOperation, Expression, ExpressionKind, UnaryOperation},
    values::{Value, ValueKind},
    Ast,
};
use tanitc_ident::Name;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Ast, Message> {
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let expr = match next.lexem_ref() {
            Lexem::Plus
            | Lexem::Minus
            | Lexem::Ampersand
            | Lexem::Star
            | Lexem::Not
            | Lexem::LParen => self.parse_factor()?,
            _ => self.parse_assign()?,
        };

        self.set_ignore_nl_option(old_opt);

        if let Ast::Expression(node) = &expr {
            let location = node.location.clone();

            if let ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } = &node.kind
            {
                let new_op = match operation {
                    BinaryOperation::AddAssign => Some(Lexem::Plus),
                    BinaryOperation::SubAssign => Some(Lexem::Minus),
                    BinaryOperation::MulAssign => Some(Lexem::Star),
                    BinaryOperation::DivAssign => Some(Lexem::Slash),
                    BinaryOperation::ModAssign => Some(Lexem::Percent),
                    BinaryOperation::BitwiseXorAssign => Some(Lexem::Xor),
                    BinaryOperation::BitwiseAndAssign => Some(Lexem::Ampersand),
                    BinaryOperation::BitwiseOrAssign => Some(Lexem::Stick),
                    BinaryOperation::BitwiseShiftLAssign => Some(Lexem::LShift),
                    BinaryOperation::BitwiseShiftRAssign => Some(Lexem::RShift),
                    _ => None,
                };

                if let Some(new_op) = new_op {
                    return Ok(Ast::from(Expression {
                        location: location.clone(),
                        kind: ExpressionKind::new_binary(
                            Lexem::Assign,
                            lhs.clone(),
                            Box::new(Ast::from(Expression {
                                location,
                                kind: ExpressionKind::new_binary(new_op, lhs.clone(), rhs.clone())?,
                            })),
                        )?,
                    }));
                }
            }
        }

        Ok(expr)
    }

    pub fn parse_factor(&mut self) -> Result<Ast, Message> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Ampersand => {
                self.get_token();

                let operation = {
                    if self.is_next(Lexem::KwMut) {
                        self.get_token();
                        UnaryOperation::RefMut
                    } else {
                        UnaryOperation::Ref
                    }
                };

                let node = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::Unary { operation, node },
                }))
            }
            Lexem::Plus | Lexem::Minus | Lexem::Star | Lexem::Not => {
                self.get_token();
                let operation = next.lexem_ref().clone();
                let node = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_unary(operation, node)?,
                }))
            }
            Lexem::Integer(_) => Ok(Ast::from(Value {
                location,
                kind: ValueKind::Integer(
                    self.consume_integer()?
                        .lexem_ref()
                        .get_int()
                        .unwrap_or_default(),
                ),
            })),

            Lexem::Decimal(_) => Ok(Ast::from(Value {
                location,
                kind: ValueKind::Decimal(
                    self.consume_decimal()?
                        .lexem_ref()
                        .get_dec()
                        .unwrap_or_default(),
                ),
            })),

            Lexem::Identifier(_) => {
                let identifier = self.consume_identifier()?;

                let old_opt = self.does_ignore_nl();
                self.set_ignore_nl_option(true);

                if self.is_next(Lexem::LParen) {
                    // if call
                    let arguments = self.parse_call_params()?;

                    self.set_ignore_nl_option(old_opt);
                    return Ok(Ast::from(Value {
                        location,
                        kind: ValueKind::Call {
                            identifier,
                            arguments,
                        },
                    }));
                } else if self.is_next(Lexem::Dcolon) {
                    // if ::
                    self.get_token();
                    self.set_ignore_nl_option(old_opt);

                    let lhs = Box::new(Ast::from(Value {
                        location: location.clone(),
                        kind: ValueKind::Identifier(identifier),
                    }));

                    let rhs = Box::new(self.parse_factor()?);

                    return Ok(Ast::from(Expression {
                        location,
                        kind: ExpressionKind::Access { lhs, rhs },
                    }));
                } else if self.is_next(Lexem::Lcb) {
                    // if struct
                    let components = self.parse_struct_value()?;

                    self.set_ignore_nl_option(old_opt);
                    return Ok(Ast::from(Value {
                        location,
                        kind: ValueKind::Struct {
                            name: Name {
                                id: identifier,
                                ..Default::default()
                            },
                            components,
                        },
                    }));
                } else if self.is_next(Lexem::Lsb) {
                    // if indexing: [i + 1]

                    let lhs = Box::new(Ast::from(Value {
                        kind: ValueKind::Identifier(identifier),
                        location,
                    }));

                    let index = Box::new(self.parse_array_indexing()?);

                    return Ok(Ast::from(Expression {
                        kind: ExpressionKind::Indexing { lhs, index },
                        location: next.location_ref().clone(),
                    }));
                }

                Ok(Ast::from(Value {
                    location,
                    kind: ValueKind::Identifier(identifier),
                }))
            }

            Lexem::LParen => {
                self.consume_token(Lexem::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if self.is_next(Lexem::RParen) {
                    self.get_token();
                    return Ok(Ast::from(Value {
                        location,
                        kind: ValueKind::Tuple {
                            components: Vec::new(),
                        },
                    }));
                }

                let mut components = Vec::<Ast>::new();

                let expr = self.parse_expression()?;

                let is_tuple = match &expr {
                    Ast::Expression { .. } => false,
                    Ast::Value { .. } => true,
                    _ => return Err(Message::new(next.location_ref(), "Unexpected node parsed")),
                };

                /* If parsed one expression, we return expression */
                if !is_tuple {
                    self.consume_token(Lexem::RParen)?;
                    return Ok(expr);
                }

                /* else try parse tuple */
                components.push(expr);

                loop {
                    let Some(next) = self.peek_token() else {
                        break;
                    };

                    if *next.lexem_ref() == Lexem::RParen {
                        self.consume_token(Lexem::RParen)?;
                        break;
                    } else if *next.lexem_ref() == Lexem::Comma {
                        self.consume_token(Lexem::Comma)?;
                        components.push(self.parse_expression()?);
                    } else {
                        return Err(Message::from_string(
                            next.location_ref(),
                            format!("Unexpected token \"{next}\" within tuple"),
                        ));
                    }
                }

                Ok(Ast::from(Value {
                    location,
                    kind: ValueKind::Tuple { components },
                }))
            }

            Lexem::Lsb => self.parse_array_value(),

            _ => Err(Message::from_string(
                next.location_ref(),
                format!("Unexpected token \"{next}\" within expression"),
            )),
        }
    }

    // [index]
    pub fn parse_array_indexing(&mut self) -> Result<Ast, Message> {
        self.consume_token(Lexem::Lsb)?;
        let index = self.parse_expression()?;
        self.consume_token(Lexem::Rsb)?;

        Ok(index)
    }
}

impl Parser {
    fn parse_assign(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_or()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Assign
            | Lexem::AddAssign
            | Lexem::SubAssign
            | Lexem::MulAssign
            | Lexem::DivAssign
            | Lexem::ModAssign
            | Lexem::OrAssign
            | Lexem::AndAssign
            | Lexem::XorAssign
            | Lexem::LShiftAssign
            | Lexem::RShiftAssign => {
                self.get_token();
                let operation = next.lexem_ref().clone();

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_or(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_and()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Or => {
                self.get_token();
                let operation = Lexem::Or;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_and(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_bitwise_or()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::And => {
                self.get_token();
                let operation = Lexem::And;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_or(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_bitwise_xor()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Stick => {
                self.get_token();
                let operation = Lexem::Stick;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_xor(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_bitwise_and()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Xor => {
                self.get_token();
                let operation = Lexem::Xor;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_and(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_eq()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Ampersand => {
                self.get_token();
                let operation = Lexem::Ampersand;

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_eq(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_logical_less_or_greater()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Eq | Lexem::Neq => {
                self.get_token();
                let operation = next.lexem_ref().clone();

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_less_or_greater(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_shift()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
                self.get_token();
                let operation = next.lexem_ref().clone();

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_shift(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_add_or_sub()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::LShift | Lexem::RShift => {
                self.get_token();
                let operation = next.lexem_ref().clone();

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_add_or_sub(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_mul_or_div()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Plus | Lexem::Minus => {
                self.get_token();
                let operation = next.lexem_ref().clone();

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_mul_or_div(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_dot_or_as()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        let location = next.location_ref().clone();

        match next.lexem_ref() {
            Lexem::Star | Lexem::Slash | Lexem::Percent => {
                self.get_token();
                let operation = next.lexem_ref().clone();

                let rhs = Box::new(self.parse_expression()?);

                Ok(Ast::from(Expression {
                    location,
                    kind: ExpressionKind::new_binary(operation, Box::new(lhs), rhs)?,
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_dot_or_as(&mut self) -> Result<Ast, Message> {
        let lhs = self.parse_factor()?;

        let next = self.peek_token().ok_or(Message::reached_eof())?;
        match next.lexem_ref() {
            Lexem::Dot | Lexem::KwAs => {
                self.get_token();
                let operation = next.lexem_ref().clone();
                let is_conversion = Lexem::KwAs == operation;

                if is_conversion {
                    let location = next.location_ref().clone();
                    return Ok(Ast::from(Expression {
                        location,
                        kind: ExpressionKind::Conversion {
                            lhs: Box::new(lhs),
                            ty: self.parse_type_spec()?,
                        },
                    }));
                }

                Ok(Ast::from(Expression {
                    location: next.location_ref().clone(),
                    kind: ExpressionKind::new_binary(
                        operation,
                        Box::new(lhs),
                        Box::new(self.parse_expression()?),
                    )?,
                }))
            }
            _ => Ok(lhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use tanitc_ast::ast::{
        expressions::{BinaryOperation, Expression, ExpressionKind},
        values::{Value, ValueKind},
        Ast,
    };
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    use crate::Parser;

    fn get_location(col: usize) -> Location {
        Location {
            col,
            path: PathBuf::from("text").into(),
            ..Default::default()
        }
    }

    #[test]
    fn binary_expression_test() {
        use tanitc_ident::Ident;

        const SRC_TEXT: &str = "a += 1 * 4 / (1 + a) == 3\n";

        let a_id = Ident::from("a".to_string());

        let expected = Ast::from(Expression {
            location: get_location(4),
            kind: ExpressionKind::Binary {
                operation: BinaryOperation::Assign,
                lhs: Box::new(Ast::from(Value {
                    location: get_location(1),
                    kind: ValueKind::Identifier(a_id),
                })),
                rhs: Box::new(Ast::from(Expression {
                    location: get_location(4),
                    kind: ExpressionKind::Binary {
                        operation: BinaryOperation::Add,
                        lhs: Box::new(Ast::from(Value {
                            location: get_location(1),
                            kind: ValueKind::Identifier(a_id),
                        })),
                        rhs: Box::new(Ast::from(Expression {
                            location: get_location(8),
                            kind: ExpressionKind::Binary {
                                operation: BinaryOperation::Mul,
                                lhs: Box::new(Ast::from(Value {
                                    location: get_location(6),
                                    kind: ValueKind::Integer(1),
                                })),
                                rhs: Box::new(Ast::from(Expression {
                                    location: get_location(23),
                                    kind: ExpressionKind::Binary {
                                        operation: BinaryOperation::LogicalEq,
                                        lhs: Box::new(Ast::from(Expression {
                                            location: get_location(12),
                                            kind: ExpressionKind::Binary {
                                                operation: BinaryOperation::Div,
                                                lhs: Box::new(Ast::from(Value {
                                                    location: get_location(10),
                                                    kind: ValueKind::Integer(4),
                                                })),
                                                rhs: Box::new(Ast::from(Expression {
                                                    location: get_location(17),
                                                    kind: ExpressionKind::Binary {
                                                        operation: BinaryOperation::Add,
                                                        lhs: Box::new(Ast::from(Value {
                                                            location: get_location(15),
                                                            kind: ValueKind::Integer(1),
                                                        })),
                                                        rhs: Box::new(Ast::from(Value {
                                                            location: get_location(19),
                                                            kind: ValueKind::Identifier(a_id),
                                                        })),
                                                    },
                                                })),
                                            },
                                        })),
                                        rhs: Box::new(Ast::from(Value {
                                            location: get_location(25),
                                            kind: ValueKind::Integer(3),
                                        })),
                                    },
                                })),
                            },
                        })),
                    },
                })),
            },
        });

        let mut parser = Parser::from_text(SRC_TEXT);
        let ast = parser.parse_expression().unwrap();

        assert_eq!(ast, expected);
    }

    #[test]
    fn conversion_test() {
        const SRC_TEXT: &str = "45 as f32\n";

        let mut parser = Parser::from_text(SRC_TEXT);

        let expr = parser.parse_expression().unwrap();
        let Ast::Expression(node) = expr else {
            panic!("Expected Ast::Expression, actually: {}", expr.name());
        };

        let ExpressionKind::Conversion { lhs, ty } = &node.kind else {
            panic!("Expected ExpressionKind::Conversion");
        };

        assert!(matches!(
            lhs.as_ref(),
            Ast::Value(Value {
                kind: ValueKind::Integer(45),
                ..
            })
        ));

        assert_eq!(ty.get_type(), Type::F32);
    }
}
