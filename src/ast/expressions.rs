use crate::analyzer::SymbolData;
use crate::ast::{
    functions::FunctionNode, identifiers::Identifier, types, values, variables::VariableNode, Ast,
    IAst, Stream,
};
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::error_listener::{
    MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR, UNEXPECTED_NODE_PARSED_ERROR_STR,
    UNEXPECTED_TOKEN_ERROR_STR,
};
use crate::lexer::{self, Lexem};
use crate::parser::{put_intent, Parser};

use std::io::Write;
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub enum Expression {
    Unary {
        operation: Lexem,
        node: Box<Ast>,
    },
    Binary {
        operation: Lexem,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
}

impl Expression {
    pub fn parse(parser: &mut Parser) -> Result<Ast, &'static str> {
        let old_opt = parser.does_ignore_nl();

        parser.set_ignore_nl_option(false);
        let res = Self::parse_assign(parser);
        parser.set_ignore_nl_option(old_opt);

        res
    }

    pub fn parse_factor(parser: &mut Parser) -> Result<Ast, &'static str> {
        let next = parser.peek_token();

        match &next.lexem {
            Lexem::Plus | Lexem::Minus | Lexem::Ampersand | Lexem::Star | Lexem::Not => {
                parser.get_token();
                let operation = next.lexem;
                let node = Box::new(Self::parse(parser)?);

                Ok(Ast::Expression {
                    node: Box::new(Expression::Unary { operation, node }),
                })
            }
            Lexem::Integer(_) => Ok(Ast::Value {
                node: values::Value::Integer(parser.consume_integer()?),
            }),

            Lexem::Decimal(_) => Ok(Ast::Value {
                node: values::Value::Decimal(parser.consume_decimal()?),
            }),

            Lexem::Identifier(_) => {
                let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

                let next = parser.peek_token();
                if next.lexem == Lexem::LParen {
                    // if call
                    let arguments = values::Value::parse_call(parser)?;

                    return Ok(Ast::Value {
                        node: values::Value::Call {
                            identifier,
                            arguments,
                        },
                    });
                } else if next.lexem == Lexem::Dcolon {
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
                } else if next.lexem == Lexem::Lcb {
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

            Lexem::LParen => {
                parser.consume_token(Lexem::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if parser.peek_token().lexem == Lexem::RParen {
                    parser.consume_token(Lexem::RParen)?;
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

            Lexem::Lsb => values::Value::parse_array(parser),

            _ => {
                parser.error(
                    &format!("Unexpected token \"{}\" within expression", next),
                    next.get_location(),
                );

                Err(UNEXPECTED_TOKEN_ERROR_STR)
            }
        }
    }

    pub fn convert_ast_node(
        expr_node: &mut Ast,
        analyzer: &mut crate::analyzer::Analyzer,
    ) -> Result<(), &'static str> {
        if let Ast::Expression { node } = expr_node {
            if let Expression::Binary {
                operation,
                lhs,
                rhs,
            } = node.as_ref()
            {
                let is_conversion = *operation == Lexem::KwAs;

                let lhs_type = lhs.get_type(analyzer);

                if !is_conversion {
                    let rhs_type = rhs.get_type(analyzer);
                    let func_id = Identifier::Common(format!(
                        "__tanit_compiler__{}_{}_{}",
                        match operation {
                            Lexem::Plus => "add",
                            Lexem::Minus => "sub",
                            Lexem::Star => "mul",
                            Lexem::Slash => "div",
                            Lexem::Percent => "mod",
                            _ => return Err("Unexpected operation"),
                        },
                        lhs_type.clone(),
                        rhs_type.clone()
                    ));

                    *expr_node = Ast::FuncDef {
                        node: FunctionNode {
                            identifier: func_id,
                            return_type: lhs_type.clone(),
                            parameters: vec![
                                Ast::VariableDef {
                                    node: VariableNode {
                                        identifier: Identifier::from_str("_A")?,
                                        var_type: lhs_type.clone(),
                                        is_global: false,
                                        is_mutable: false,
                                    },
                                },
                                Ast::VariableDef {
                                    node: VariableNode {
                                        identifier: Identifier::from_str("_B")?,
                                        var_type: rhs_type,
                                        is_global: false,
                                        is_mutable: false,
                                    },
                                },
                            ],
                            body: None,
                        },
                    }
                } else {
                    let rhs_type = if let Ast::Value {
                        node: values::Value::Identifier(id),
                    } = rhs.as_ref()
                    {
                        types::Type::from_id(id)
                    } else {
                        types::Type::new()
                    };
                    *expr_node = Ast::Expression {
                        node: Box::new(Expression::Binary {
                            operation: Lexem::KwAs,
                            lhs: lhs.clone(),
                            rhs: Box::new(Ast::TypeDecl { node: rhs_type }),
                        }),
                    };
                };
            }
            Ok(())
        } else {
            Err("Expected \"Expression\"")
        }
    }
}

impl Expression {
    fn parse_assign(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_logical_or(parser)?;

        let next = parser.peek_token();
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
            Lexem::Or => {
                parser.get_token();
                let operation = Lexem::Or;

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
            Lexem::And => {
                parser.get_token();
                let operation = Lexem::And;

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
            Lexem::Stick => {
                parser.get_token();
                let operation = Lexem::Stick;

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
            Lexem::Xor => {
                parser.get_token();
                let operation = Lexem::Xor;

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
            Lexem::Ampersand => {
                parser.get_token();
                let operation = Lexem::Ampersand;

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
            Lexem::Eq | Lexem::Neq => {
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
            Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
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
            Lexem::LShift | Lexem::RShift => {
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
            Lexem::Plus | Lexem::Minus => {
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
        let lhs = Self::parse_dot_or_as(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            Lexem::Star | Lexem::Slash | Lexem::Percent => {
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

    fn parse_dot_or_as(parser: &mut Parser) -> Result<Ast, &'static str> {
        let lhs = Self::parse_factor(parser)?;

        let next = parser.peek_token();
        match next.lexem {
            Lexem::Dot | Lexem::KwAs => {
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
}

impl IAst for Expression {
    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> types::Type {
        match self {
            Self::Binary {
                operation,
                lhs,
                rhs,
            } => match operation {
                Lexem::Neq | Lexem::Eq | Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
                    types::Type::Bool
                }

                _ => {
                    let is_conversion = *operation == Lexem::KwAs;

                    let rhs_type = if is_conversion {
                        if let Ast::Value {
                            node: values::Value::Identifier(id),
                        } = rhs.as_ref()
                        {
                            types::Type::from_id(id)
                        } else {
                            analyzer.error("rhs expected to be a type");
                            types::Type::new()
                        }
                    } else {
                        return rhs.get_type(analyzer);
                    };

                    if is_conversion {
                        return rhs_type;
                    }

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

                    analyzer.error(&format!(
                        "Mismatched types {:?} and {:?}",
                        lhs_type, rhs_type
                    ));

                    types::Type::new()
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
                let is_conversion = *operation == Lexem::KwAs;

                let mut lhs_type: types::Type = lhs.get_type(analyzer);

                let rhs_type = if is_conversion {
                    if let Ast::TypeDecl { node } = rhs.as_ref() {
                        node.clone()
                    } else {
                        return Err("rhs must be a type");
                    }
                } else {
                    let t = rhs.get_type(analyzer);
                    rhs.analyze(analyzer)?;
                    t
                };

                if *operation == Lexem::Assign
                    || *operation == Lexem::SubAssign
                    || *operation == Lexem::AddAssign
                    || *operation == Lexem::DivAssign
                    || *operation == Lexem::ModAssign
                    || *operation == Lexem::MulAssign
                    || *operation == Lexem::AndAssign
                    || *operation == Lexem::OrAssign
                    || *operation == Lexem::XorAssign
                    || *operation == Lexem::LShiftAssign
                    || *operation == Lexem::RShiftAssign
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
                                &format!("Variable \"{}\" defined with type \"{:?}\", but is assigned to \"{:?}\"",
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
                        match node {
                            values::Value::Identifier(id) => {
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
                            }
                            values::Value::Integer(..) | values::Value::Decimal(..) => {}
                            values::Value::Text(..) => {
                                analyzer.error("Cannot perform operation with text in this context")
                            }
                            values::Value::Array { .. } => analyzer
                                .error("Cannot perform operation with array in this context"),
                            values::Value::Tuple { .. } => analyzer
                                .error("Cannot perform operation with tuple in this context"),
                            _ => analyzer.error("Cannot perform operation with this object"),
                        }
                    } else {
                        lhs.analyze(analyzer)?;
                    }
                } else {
                    lhs.analyze(analyzer)?;
                }

                if lhs_type != rhs_type {
                    if is_conversion {
                        if !lhs_type.is_common() || !rhs_type.is_common() {
                            analyzer
                                .error(&format!("Cannot cast {:?} to {:?}", lhs_type, rhs_type));
                        }
                    } else {
                        analyzer.error(&format!(
                            "Cannot perform operation with objects with different types: {:?} and {:?}",
                            lhs_type, rhs_type
                        ));
                    }
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

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        match self {
            Expression::Unary { operation, node } => {
                write!(stream, "{}", operation)?;
                node.codegen(stream)?;
            }
            Expression::Binary {
                operation: lexer::Lexem::Assign,
                lhs,
                rhs,
            } => {
                lhs.codegen(stream)?;
                write!(stream, " = ")?;
                rhs.codegen(stream)?;
            }
            Expression::Binary {
                operation: lexer::Lexem::KwAs,
                lhs,
                rhs,
            } => {
                write!(stream, "((")?;
                rhs.codegen(stream)?;
                write!(stream, ")")?;
                lhs.codegen(stream)?;
                write!(stream, ")")?;
            }
            _ => unreachable!(),
        }

        stream.mode = old_mode;
        Ok(())
    }
}
