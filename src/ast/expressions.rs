use crate::analyzer::SymbolData;
use crate::ast::{types, values, Ast, IAst, Stream};
use crate::error_listener::{
    MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR, UNEXPECTED_NODE_PARSED_ERROR_STR,
    UNEXPECTED_TOKEN_ERROR_STR,
};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::Parser;

use std::io::Write;

#[derive(Clone, PartialEq)]
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

impl Expression {
    pub fn parse(parser: &mut Parser) -> Result<Ast, &'static str> {
        Self::parse_assign(parser)
    }

    fn parse_assign(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_logical_or(parser)?;

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

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_or(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_logical_and(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Or => {
                parser.get_token();
                let operation = TokenType::Or;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_and(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_bitwise_or(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::And => {
                parser.get_token();
                let operation = TokenType::And;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_or(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_bitwise_xor(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Stick => {
                parser.get_token();
                let operation = TokenType::Stick;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_xor(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_bitwise_and(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Xor => {
                parser.get_token();
                let operation = TokenType::Xor;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_bitwise_and(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_logical_eq(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Ampersand => {
                parser.get_token();
                let operation = TokenType::Ampersand;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_eq(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_logical_less_or_greater(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Eq | TokenType::Neq => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_logical_less_or_greater(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_shift(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Lt | TokenType::Lte | TokenType::Gt | TokenType::Gte => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_shift(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_add_or_sub(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::LShift | TokenType::RShift => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_add_or_sub(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_mul_or_div(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Plus | TokenType::Minus => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_mul_or_div(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_dot(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Star | TokenType::Slash | TokenType::Percent => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_dot(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_factor(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            TokenType::Dot => {
                parser.get_token();
                let operation = next.lexem;

                let rhs = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Binary {
                        operation,
                        lhs: Box::new(lhs),
                        rhs,
                    }),
                })
            }

            _ => Ok(lhs),
        }
    }

    fn parse_factor(parser: &mut Parser) -> Result<Ast, &'static str> {
        let next = parser.peek_token();

        match &next.lexem {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Ampersand
            | TokenType::Star
            | TokenType::Not => {
                parser.get_token();
                let operation = next.lexem;
                let node = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Unary { operation, node }),
                })
            }
            TokenType::Integer(_) => Ok(Ast::Value {
                node: values::Value::Integer(parser.consume_integer()?),
            }),

            TokenType::Decimal(_) => Ok(Ast::Value {
                node: values::Value::Decimal(parser.consume_decimal()?),
            }),

            TokenType::Identifier(_) => {
                let identifier = parser.consume_identifier()?;

                let next = parser.peek_token();
                if next.lexem == TokenType::LParen {
                    // if call
                    let arguments = values::Value::parse_call(parser)?;

                    return Ok(Ast::Value {
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

                    let rhs = Box::new(Self::parse_factor(parser)?);

                    return Ok(Ast::Expression {
                        node: Box::new(Expression::Binary {
                            operation,
                            lhs,
                            rhs,
                        }),
                    });
                } else if next.lexem == TokenType::Lcb {
                    // if struct
                    let components = values::Value::parse_struct(parser)?;

                    return Ok(Ast::Value {
                        node: values::Value::Struct {
                            identifier,
                            components,
                        },
                    });
                }

                Ok(Ast::Value {
                    node: values::Value::Identifier(identifier),
                })
            }

            TokenType::LParen => {
                parser.consume_token(TokenType::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if parser.peek_token().lexem == TokenType::RParen {
                    parser.consume_token(TokenType::RParen)?;
                    return Ok(Ast::Value {
                        node: values::Value::Tuple {
                            components: Vec::new(),
                        },
                    });
                }

                let mut components = Vec::<Ast>::new();

                let expr = Self::parse(parser)?;

                let is_tuple = match &expr {
                    Ast::Expression { .. } => false,
                    Ast::Value { .. } => true,
                    _ => {
                        parser.error("Unexpected node parsed", next.get_location());
                        return Err(UNEXPECTED_NODE_PARSED_ERROR_STR);
                    }
                };

                /* If parsed one expression, we return expression */
                if !is_tuple {
                    parser.consume_token(TokenType::RParen)?;
                    return Ok(expr);
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
                        components.push(Self::parse(parser)?);
                    } else {
                        parser.error(
                            &format!("Unexpected token \"{}\" within tuple", next),
                            next.get_location(),
                        );
                        return Err(UNEXPECTED_TOKEN_ERROR_STR);
                    }
                }

                Ok(Ast::Value {
                    node: values::Value::Tuple { components },
                })
            }

            TokenType::Lsb => values::Value::parse_array(parser),

            _ => {
                parser.error(
                    &format!("Unexpected token \"{}\" within expression", next),
                    next.get_location(),
                );

                Err(UNEXPECTED_TOKEN_ERROR_STR)
            }
        }
    }
}

impl IAst for Expression {
    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> types::Type {
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
                | TokenType::Gte => types::Type::Bool,

                _ => {
                    let rhs_type = rhs.get_type(analyzer);

                    let mut lhs_type = if let Ast::VariableDef { node } = lhs.as_ref() {
                        node.var_type.clone()
                    } else {
                        lhs.get_type(analyzer)
                    };

                    if let types::Type::Custom(t) = &mut lhs_type {
                        if t == "@auto" {
                            lhs_type = rhs_type.clone();
                        }
                    }

                    if lhs_type == rhs_type {
                        return lhs_type;
                    }

                    analyzer.error("mismatched types");

                    types::Type::Tuple {
                        components: Vec::new(),
                    }
                }
            },
            Self::Unary { node, .. } => node.get_type(analyzer),
        }
    }

    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        match self {
            Self::Binary {
                operation,
                lhs,
                rhs,
            } => {
                let mut lhs_type = lhs.get_type(analyzer);
                let rhs_type = rhs.get_type(analyzer);

                rhs.analyze(analyzer)?;

                if *operation == TokenType::Assign
                    || *operation == TokenType::SubAssign
                    || *operation == TokenType::AddAssign
                    || *operation == TokenType::DivAssign
                    || *operation == TokenType::ModAssign
                    || *operation == TokenType::MulAssign
                    || *operation == TokenType::AndAssign
                    || *operation == TokenType::OrAssign
                    || *operation == TokenType::XorAssign
                    || *operation == TokenType::LShiftAssign
                    || *operation == TokenType::RShiftAssign
                {
                    if let Ast::VariableDef { node } = lhs.as_mut() {
                        if analyzer
                            .check_identifier_existance(&node.identifier)
                            .is_ok()
                        {
                            analyzer.error(&format!(
                                "Identifier \"{}\" defined multiple times",
                                &node.identifier
                            ));
                            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
                        }

                        if let types::Type::Custom(t) = &node.var_type {
                            if "@auto" == t {
                                node.var_type = rhs_type.clone();
                                lhs_type = rhs_type.clone();
                            }
                        }

                        if node.var_type != rhs_type {
                            analyzer.error(
                                &format!("Variable \"{:?}\" defined with type \"{:?}\", but is assigned to \"{:?}\"",
                                    node.identifier, node.var_type, rhs_type));
                        }

                        analyzer.add_symbol(
                            &node.identifier,
                            analyzer.create_symbol(SymbolData::VariableDef {
                                var_type: node.var_type.clone(),
                                is_mutable: node.is_mutable,
                                is_initialization: true,
                            }),
                        );
                    } else if let Ast::Value { node } = lhs.as_mut() {
                        if let values::Value::Identifier(id) = node {
                            if let Ok(s) = analyzer.check_identifier_existance(id) {
                                if let SymbolData::VariableDef { is_mutable, .. } = &s.data {
                                    if !*is_mutable {
                                        analyzer.error(&format!(
                                            "Variable \"{}\" is immutable in current scope",
                                            id
                                        ));
                                    }
                                }
                            }
                        } else {
                            analyzer.error("Cannot perform operation with this object");
                        }
                    } else {
                        lhs.analyze(analyzer)?;
                    }
                } else {
                    lhs.analyze(analyzer)?;
                }

                if lhs_type != rhs_type {
                    analyzer.error(&format!(
                        "Cannot perform operation with objects with different types: {:?} and {:?}",
                        lhs_type, rhs_type
                    ));
                }

                Ok(())
            }
            Self::Unary { node, .. } => node.analyze(analyzer),
        }
    }

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
