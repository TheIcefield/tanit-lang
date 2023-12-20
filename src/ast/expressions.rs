use crate::ast::{types, values, Ast, GetType, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::Parser;

use std::io::Write;

#[derive(Clone)]
pub enum Expression {
    Unary {
        operation: TokenType,
        node: Box<Ast>,
    },
    Binary {
        operation: TokenType,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
}

impl IAst for Expression {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Unary { operation, node } => {
                writeln!(
                    stream,
                    "{}<operation style=\"unary\" operator=\"{}\">",
                    put_intent(intent),
                    operation
                )?;

                node.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</operation>", put_intent(intent))?;
            }
            Self::Binary {
                operation,
                lhs,
                rhs,
            } => {
                writeln!(
                    stream,
                    "{}<operation style=\"binary\" operator=\"{}\">",
                    put_intent(intent),
                    operation
                )?;

                lhs.traverse(stream, intent + 1)?;

                rhs.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</operation>", put_intent(intent))?;
            }
        }

        Ok(())
    }
}

impl GetType for Expression {
    fn get_type(&self) -> Option<types::Type> {
        match self {
            Self::Binary {
                operation,
                lhs,
                rhs,
            } => match operation {
                TokenType::Neq
                | TokenType::Eq
                | TokenType::Lt
                | TokenType::Lte
                | TokenType::Gt
                | TokenType::Gte => Some(types::Type::Bool),

                _ => {
                    let lhs_type = lhs.get_type();
                    let rhs_type = rhs.get_type();

                    if lhs_type == rhs_type {
                        return lhs_type;
                    }

                    None
                }
            },
            Self::Unary { node, .. } => node.get_type(),
        }
    }
}

pub fn parse_expression(parser: &mut Parser) -> Option<Ast> {
    parse_assign(parser)
}

fn parse_assign(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_or(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Assign
        | TokenType::AddAssign
        | TokenType::SubAssign
        | TokenType::MulAssign
        | TokenType::DivAssign
        | TokenType::ModAssign
        | TokenType::OrAssign
        | TokenType::AndAssign
        | TokenType::XorAssign
        | TokenType::LShiftAssign
        | TokenType::RShiftAssign => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_logical_or(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_and(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Or => {
            parser.get_token();
            let operation = TokenType::Or;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_logical_and(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_bitwise_or(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::And => {
            parser.get_token();
            let operation = TokenType::And;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_bitwise_or(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_bitwise_xor(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Stick => {
            parser.get_token();
            let operation = TokenType::Stick;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_bitwise_xor(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_bitwise_and(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Xor => {
            parser.get_token();
            let operation = TokenType::Xor;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_bitwise_and(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_eq(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Ampersand => {
            parser.get_token();
            let operation = TokenType::Ampersand;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_logical_eq(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_less_or_greater(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Eq | TokenType::Neq => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_logical_less_or_greater(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_shift(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Lt | TokenType::Lte | TokenType::Gt | TokenType::Gte => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_shift(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_add_or_sub(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::LShift | TokenType::RShift => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_add_or_sub(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_mul_or_div(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Plus | TokenType::Minus => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_mul_or_div(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_dot(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Star | TokenType::Slash | TokenType::Percent => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_dot(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_factor(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Dot => {
            parser.get_token();
            let operation = next.lexem;

            let rhs = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Binary {
                    operation,
                    lhs: Box::new(lhs),
                    rhs,
                }),
            })
        }

        _ => Some(lhs),
    }
}

fn parse_factor(parser: &mut Parser) -> Option<Ast> {
    let next = parser.peek_token();

    match next.lexem {
        TokenType::Plus
        | TokenType::Minus
        | TokenType::Ampersand
        | TokenType::Star
        | TokenType::Not => {
            parser.get_token();
            let operation = next.lexem;
            let node = Box::new(parse_expression(parser)?);

            Some(Ast::Expression {
                node: Box::new(Expression::Unary { operation, node }),
            })
        }
        TokenType::Integer(val) => {
            parser.get_token();
            Some(Ast::Value {
                node: values::Value::Integer(val),
            })
        }

        TokenType::Decimal(val) => {
            parser.get_token();
            Some(Ast::Value {
                node: values::Value::Decimal(val),
            })
        }

        TokenType::Identifier(identifier) => {
            parser.get_token();

            let next = parser.peek_token();
            if next.lexem == TokenType::LParen {
                // if call
                let arguments = values::Value::parse_call(parser)?;

                return Some(Ast::Value {
                    node: values::Value::Call {
                        identifier,
                        arguments,
                    },
                });
            } else if next.lexem == TokenType::Dcolon {
                // if ::
                parser.get_token();

                let operation = next.lexem;

                let lhs = Box::new(Ast::Value {
                    node: values::Value::Identifier(identifier),
                });

                let rhs = Box::new(parse_factor(parser)?);

                return Some(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs,
                        rhs,
                    }),
                });
            } else if next.lexem == TokenType::Lcb {
                // if struct
                let components = values::Value::parse_struct(parser)?;

                return Some(Ast::Value {
                    node: values::Value::Struct {
                        identifier,
                        components,
                    },
                });
            }

            Some(Ast::Value {
                node: values::Value::Identifier(identifier),
            })
        }

        TokenType::LParen => {
            parser.consume_token(TokenType::LParen)?;

            /* If parsed `()` then we return empty tuple */
            if parser.peek_token().lexem == TokenType::RParen {
                parser.consume_token(TokenType::RParen)?;
                return Some(Ast::Value {
                    node: values::Value::Tuple {
                        components: Vec::new(),
                    },
                });
            }

            let mut components = Vec::<Ast>::new();

            let expr = parse_expression(parser)?;

            let is_tuple = match &expr {
                Ast::Expression { .. } => false,
                Ast::Value { .. } => true,
                _ => {
                    parser.error("Unexpected node parsed", next.get_location());
                    return None;
                }
            };

            /* If parsed one expression, we return expression */
            if !is_tuple {
                parser.consume_token(TokenType::RParen)?;
                return Some(expr);
            }

            /* else try parse tuple */
            components.push(expr);

            loop {
                let next = parser.peek_token();

                if next.lexem == TokenType::RParen {
                    parser.consume_token(TokenType::RParen)?;
                    break;
                } else if next.lexem == TokenType::Comma {
                    parser.consume_token(TokenType::Comma)?;
                    components.push(parse_expression(parser)?);
                } else {
                    parser.error(
                        &format!("Unexpected token \"{}\" within tuple", next),
                        next.get_location(),
                    );
                    return None;
                }
            }

            Some(Ast::Value {
                node: values::Value::Tuple { components },
            })
        }

        TokenType::Lsb => values::Value::parse_array(parser),

        _ => {
            parser.error(
                &format!("Unexpected token \"{}\" within expression", next),
                next.get_location(),
            );

            None
        }
    }
}
