use super::{Expression, ExpressionType};
use crate::ast::{
    types::TypeSpec,
    values::{Value, ValueType},
    Ast,
};

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_parser::Parser;

impl Expression {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let expr = Self::parse_assign(parser)?;
        parser.set_ignore_nl_option(old_opt);

        if let Ast::Expression(node) = &expr {
            let location = node.location;

            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &node.expr
            {
                let new_op = match operation {
                    Lexem::AddAssign => Some(Lexem::Plus),
                    Lexem::SubAssign => Some(Lexem::Minus),
                    Lexem::MulAssign => Some(Lexem::Star),
                    Lexem::DivAssign => Some(Lexem::Slash),
                    Lexem::ModAssign => Some(Lexem::Percent),
                    Lexem::XorAssign => Some(Lexem::Xor),
                    Lexem::AndAssign => Some(Lexem::Ampersand),
                    Lexem::OrAssign => Some(Lexem::Stick),
                    Lexem::LShiftAssign => Some(Lexem::LShift),
                    Lexem::RShiftAssign => Some(Lexem::RShift),
                    _ => None,
                };

                if let Some(new_op) = new_op {
                    return Ok(Ast::from(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation: Lexem::Assign,
                            lhs: lhs.clone(),
                            rhs: Box::new(Ast::from(Self {
                                location,
                                expr: ExpressionType::Binary {
                                    operation: new_op,
                                    lhs: lhs.clone(),
                                    rhs: rhs.clone(),
                                },
                            })),
                        },
                    }));
                }
            }
        }

        Ok(expr)
    }

    pub fn parse_factor(parser: &mut Parser) -> Result<Ast, Message> {
        let next = parser.peek_token();
        let location = next.location;

        match &next.lexem {
            Lexem::Plus | Lexem::Minus | Lexem::Ampersand | Lexem::Star | Lexem::Not => {
                parser.get_token();
                let operation = next.lexem;
                let node = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Unary { operation, node },
                }))
            }
            Lexem::Integer(_) => Ok(Ast::from(Value {
                location,
                value: ValueType::Integer(
                    parser
                        .consume_integer()?
                        .lexem
                        .get_int()
                        .unwrap_or_default(),
                ),
            })),

            Lexem::Decimal(_) => Ok(Ast::from(Value {
                location,
                value: ValueType::Decimal(
                    parser
                        .consume_decimal()?
                        .lexem
                        .get_dec()
                        .unwrap_or_default(),
                ),
            })),

            Lexem::Identifier(_) => {
                let identifier = parser.consume_identifier()?;

                let next = parser.peek_token();
                if next.lexem == Lexem::LParen {
                    // if call
                    let arguments = Value::parse_call_params(parser)?;

                    return Ok(Ast::from(Value {
                        location,
                        value: ValueType::Call {
                            identifier,
                            arguments,
                        },
                    }));
                } else if next.lexem == Lexem::Dcolon {
                    // if ::
                    parser.get_token();

                    let operation = next.lexem;

                    let lhs = Box::new(Ast::from(Value {
                        location,
                        value: ValueType::Identifier(identifier),
                    }));

                    let rhs = Box::new(Self::parse_factor(parser)?);

                    return Ok(Ast::from(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs,
                            rhs,
                        },
                    }));
                } else if next.lexem == Lexem::Lcb {
                    // if struct
                    let components = Value::parse_struct(parser)?;

                    return Ok(Ast::from(Value {
                        location,
                        value: ValueType::Struct {
                            identifier,
                            components,
                        },
                    }));
                }

                Ok(Ast::from(Value {
                    location,
                    value: ValueType::Identifier(identifier),
                }))
            }

            Lexem::LParen => {
                parser.consume_token(Lexem::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if parser.peek_token().lexem == Lexem::RParen {
                    parser.consume_token(Lexem::RParen)?;
                    return Ok(Ast::from(Value {
                        location,
                        value: ValueType::Tuple {
                            components: Vec::new(),
                        },
                    }));
                }

                let mut components = Vec::<Ast>::new();

                let expr = Self::parse(parser)?;

                let is_tuple = match &expr {
                    Ast::Expression { .. } => false,
                    Ast::Value { .. } => true,
                    _ => return Err(Message::new(next.location, "Unexpected node parsed")),
                };

                /* If parsed one expression, we return expression */
                if !is_tuple {
                    parser.consume_token(Lexem::RParen)?;
                    return Ok(expr);
                }

                /* else try parse tuple */
                components.push(expr);

                loop {
                    let next = parser.peek_token();

                    if next.lexem == Lexem::RParen {
                        parser.consume_token(Lexem::RParen)?;
                        break;
                    } else if next.lexem == Lexem::Comma {
                        parser.consume_token(Lexem::Comma)?;
                        components.push(Self::parse(parser)?);
                    } else {
                        return Err(Message::new(
                            next.location,
                            &format!("Unexpected token \"{}\" within tuple", next),
                        ));
                    }
                }

                Ok(Ast::from(Value {
                    location,
                    value: ValueType::Tuple { components },
                }))
            }

            Lexem::Lsb => Value::parse_array(parser),

            _ => Err(Message::new(
                next.location,
                &format!("Unexpected token \"{}\" within expression", next),
            )),
        }
    }
}

impl Expression {
    fn parse_assign(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_logical_or(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
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
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_or(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_logical_and(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Or => {
                parser.get_token();
                let operation = Lexem::Or;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_and(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_bitwise_or(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::And => {
                parser.get_token();
                let operation = Lexem::And;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_or(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_bitwise_xor(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Stick => {
                parser.get_token();
                let operation = Lexem::Stick;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_xor(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_bitwise_and(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Xor => {
                parser.get_token();
                let operation = Lexem::Xor;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_and(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_logical_eq(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Ampersand => {
                parser.get_token();
                let operation = Lexem::Ampersand;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_eq(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_logical_less_or_greater(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Eq | Lexem::Neq => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_less_or_greater(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_shift(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_shift(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_add_or_sub(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::LShift | Lexem::RShift => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_add_or_sub(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_mul_or_div(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Plus | Lexem::Minus => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_mul_or_div(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_dot_or_as(parser)?;

        let next = parser.peek_token();
        let location = next.location;

        match next.lexem {
            Lexem::Star | Lexem::Slash | Lexem::Percent => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::from(Self {
                    location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    },
                }))
            }

            _ => Ok(lhs),
        }
    }

    fn parse_dot_or_as(parser: &mut Parser) -> Result<Ast, Message> {
        let lhs = Self::parse_factor(parser)?;

        let next = parser.peek_token();

        match next.lexem {
            Lexem::Dot | Lexem::KwAs => {
                parser.get_token();
                let operation = next.lexem;
                let is_conversion = Lexem::KwAs == operation;

                if is_conversion {
                    let location = parser.peek_token().location;
                    return Ok(Ast::from(Self {
                        location,
                        expr: ExpressionType::Conversion {
                            lhs: Box::new(lhs),
                            ty: TypeSpec::parse(parser)?,
                        },
                    }));
                }

                Ok(Ast::from(Self {
                    location: next.location,
                    expr: ExpressionType::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs: Box::new(Self::parse(parser)?),
                    },
                }))
            }
            _ => Ok(lhs),
        }
    }
}
