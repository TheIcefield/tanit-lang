use crate::ast::{calls, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::Parser;

use std::io::Write;

use super::values;

#[derive(Clone)]
pub enum Expression {
    Unary {
        operation: TokenType,
        node: Box<Ast>,
    },
    Binary {
        operation: TokenType,
        lhs: Box<Ast>,
        rhs: Box<Ast>
    },
}

impl IAst for Expression {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Unary { operation, node } => {
                writeln!(
                    stream,
                    "{}<operation style=\"Unary\" operator=\"{}\">",
                    put_intent(intent),
                    operation
                )?;

                node.traverse(stream, intent + 1)?;

                writeln!(stream, "{}</operation>", put_intent(intent))?;
            }
            Self::Binary { operation, lhs, rhs } => {
                writeln!(
                    stream,
                    "{}<operation style=\"Binary\" operator=\"{}\">",
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

pub fn parse_expression(parser: &mut Parser) -> Option<Ast> {
    parse_assign(parser)
}

fn parse_assign(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_or(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::AddAssign
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
        TokenType::KwOr => {
            parser.get_token();
            let operation = TokenType::KwOr;

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
        TokenType::KwOr => {
            parser.get_token();
            let operation = TokenType::KwAnd;

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
        TokenType::Xor => {
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
    let lhs = parse_factor(parser)?;

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

            Some(Ast::Expression { node: Expression::Unary { operation, node } })
        }
        TokenType::Integer(val) => {
            parser.get_token();
            Some(Ast::Value {
                node: values::ValueType::Integer(val),
            })
        }

        TokenType::Decimal(val) => {
            parser.get_token();
            Some(Ast::Value {
                node: values::ValueType::Decimal(val),
            })
        }

        TokenType::Identifier(identifier) => {
            parser.get_token();

            let next = parser.peek_token();
            if next.lexem == TokenType::LParen {
                // if call
                let arguments = calls::parse_call(parser)?;

                return Some(Ast::Value {
                    node: values::ValueType::Call(calls::Node {
                        identifier,
                        arguments,
                    }),
                });
            }

            Some(Ast::Value {
                node: values::ValueType::Identifier(identifier),
            })
        }

        TokenType::LParen => {
            parser.consume_token(TokenType::LParen)?;

            let expr = parse_expression(parser)?;

            parser.consume_token(TokenType::RParen)?;

            Some(expr)
        }

        _ => {
            parser.error("Unexpected token within expression", next.get_location());

            None
        }
    }
}
