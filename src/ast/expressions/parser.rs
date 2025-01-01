use super::{Expression, ExpressionType};
use crate::ast::{
    identifiers::{Identifier, IdentifierType},
    types::Type,
    values::{CallParam, Value, ValueType},
    Ast,
};
use crate::messages::Message;
use crate::parser::{token::Lexem, Parser};

impl Expression {
    pub fn parse(parser: &mut Parser) -> Result<Ast, Message> {
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let expr = Self::parse_assign(parser)?;
        parser.set_ignore_nl_option(old_opt);

        if let Ast::Expression { node } = &expr {
            let location = node.location;

            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &node.as_ref().expr
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
                    return Ok(Ast::Expression {
                        node: Box::new(Self {
                            location,
                            expr: ExpressionType::Binary {
                                operation: Lexem::Assign,
                                lhs: lhs.clone(),
                                rhs: Box::new(Ast::Expression {
                                    node: Box::new(Self {
                                        location,
                                        expr: ExpressionType::Binary {
                                            operation: new_op,
                                            lhs: lhs.clone(),
                                            rhs: rhs.clone(),
                                        },
                                    }),
                                }),
                            },
                        }),
                    });
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Unary { operation, node },
                    }),
                })
            }
            Lexem::Integer(_) => Ok(Ast::Value {
                node: Value {
                    location,
                    value: ValueType::Integer(
                        parser
                            .consume_integer()?
                            .lexem
                            .get_int()
                            .unwrap_or_default(),
                    ),
                },
            }),

            Lexem::Decimal(_) => Ok(Ast::Value {
                node: Value {
                    location,
                    value: ValueType::Decimal(
                        parser
                            .consume_decimal()?
                            .lexem
                            .get_dec()
                            .unwrap_or_default(),
                    ),
                },
            }),

            Lexem::Identifier(_) => {
                let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                let next = parser.peek_token();
                if next.lexem == Lexem::LParen {
                    // if call
                    let arguments = Value::parse_call_params(parser)?;

                    return Ok(Ast::Value {
                        node: Value {
                            location,
                            value: ValueType::Call {
                                identifier,
                                arguments,
                            },
                        },
                    });
                } else if next.lexem == Lexem::Dcolon {
                    // if ::
                    parser.get_token();

                    let operation = next.lexem;

                    let lhs = Box::new(Ast::Value {
                        node: Value {
                            location,
                            value: ValueType::Identifier(identifier),
                        },
                    });

                    let rhs = Box::new(Self::parse_factor(parser)?);

                    return Ok(Ast::Expression {
                        node: Box::new(Self {
                            location,
                            expr: ExpressionType::Binary {
                                operation,
                                lhs,
                                rhs,
                            },
                        }),
                    });
                } else if next.lexem == Lexem::Lcb {
                    // if struct
                    let components = Value::parse_struct(parser)?;

                    return Ok(Ast::Value {
                        node: Value {
                            location,
                            value: ValueType::Struct {
                                identifier,
                                components,
                            },
                        },
                    });
                }

                Ok(Ast::Value {
                    node: Value {
                        location,
                        value: ValueType::Identifier(identifier),
                    },
                })
            }

            Lexem::LParen => {
                parser.consume_token(Lexem::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if parser.peek_token().lexem == Lexem::RParen {
                    parser.consume_token(Lexem::RParen)?;
                    return Ok(Ast::Value {
                        node: Value {
                            location,
                            value: ValueType::Tuple {
                                components: Vec::new(),
                            },
                        },
                    });
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

                Ok(Ast::Value {
                    node: Value {
                        location,
                        value: ValueType::Tuple { components },
                    },
                })
            }

            Lexem::Lsb => Value::parse_array(parser),

            _ => Err(Message::new(
                next.location,
                &format!("Unexpected token \"{}\" within expression", next),
            )),
        }
    }

    pub fn convert_ast_node(
        expr_node: &mut Ast,
        analyzer: &mut crate::analyzer::Analyzer,
    ) -> Result<(), Message> {
        if let Ast::Expression { node } = expr_node {
            let location = node.location;
            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &mut node.as_mut().expr
            {
                Self::convert_ast_node(lhs, analyzer)?;
                Self::convert_ast_node(rhs, analyzer)?;

                let is_conversion = *operation == Lexem::KwAs;

                let lhs_type = lhs.get_type(analyzer);

                if !is_conversion {
                    let rhs_type = rhs.get_type(analyzer);
                    let func_id = Identifier {
                        location,
                        identifier: IdentifierType::Common(format!(
                            "__tanit_compiler__{}_{}_{}",
                            match operation {
                                Lexem::Plus => "add",
                                Lexem::Minus => "sub",
                                Lexem::Star => "mul",
                                Lexem::Slash => "div",
                                Lexem::Percent => "mod",
                                Lexem::LShift => "lshift",
                                Lexem::RShift => "rshift",
                                Lexem::Stick => "or",
                                Lexem::Ampersand => "and",
                                _ => return Err(Message::new(location, "Unexpected operation")),
                            },
                            lhs_type,
                            rhs_type
                        )),
                    };

                    *expr_node = Ast::Value {
                        node: Value {
                            location,
                            value: ValueType::Call {
                                identifier: func_id,
                                arguments: vec![
                                    CallParam::Positional(0, lhs.clone()),
                                    CallParam::Positional(1, rhs.clone()),
                                ],
                            },
                        },
                    };
                } else {
                    let rhs_type = if let Ast::Value {
                        node:
                            Value {
                                value: ValueType::Identifier(id),
                                ..
                            },
                    } = rhs.as_ref()
                    {
                        Type::from_id(id)
                    } else {
                        Type::new()
                    };
                    *expr_node = Ast::Expression {
                        node: Box::new(Self {
                            location,
                            expr: ExpressionType::Binary {
                                operation: Lexem::KwAs,
                                lhs: lhs.clone(),
                                rhs: Box::new(Ast::Type { node: rhs_type }),
                            },
                        }),
                    };
                };
            }
            Ok(())
        } else {
            unreachable!()
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
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
                let is_conversion = operation == Lexem::KwAs;

                let mut rhs = Box::new(Self::parse(parser)?);

                if is_conversion {
                    if let Ast::Value {
                        node:
                            Value {
                                value: ValueType::Identifier(id),
                                ..
                            },
                    } = rhs.clone().as_ref()
                    {
                        rhs = Box::new(Ast::Type {
                            node: Type::from_id(id),
                        })
                    } else {
                        parser.error(Message::new(
                            next.location,
                            "Rvalue of conversion must be a type",
                        ));
                    }
                }

                Ok(Ast::Expression {
                    node: Box::new(Self {
                        location: next.location,
                        expr: ExpressionType::Binary {
                            operation,
                            lhs: Box::new(lhs),
                            rhs,
                        },
                    }),
                })
            }
            _ => Ok(lhs),
        }
    }
}
