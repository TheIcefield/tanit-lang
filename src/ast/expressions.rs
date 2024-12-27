use crate::analyzer::SymbolData;
use crate::ast::{
    identifiers::{Identifier, IdentifierType},
    types, values, Ast, IAst,
};
use crate::codegen::{CodeGenMode, CodeGenStream};
use crate::messages::Message;
use crate::parser::location::Location;
use crate::parser::{token::Lexem, Parser};
use std::io::Write;

#[derive(Clone, PartialEq)]
pub enum ExpressionType {
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

#[derive(Clone, PartialEq)]
pub struct Expression {
    pub location: Location,
    pub expr: ExpressionType,
}

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
                node: values::Value {
                    location,
                    value: values::ValueType::Integer(
                        parser
                            .consume_integer()?
                            .lexem
                            .get_int()
                            .unwrap_or_default(),
                    ),
                },
            }),

            Lexem::Decimal(_) => Ok(Ast::Value {
                node: values::Value {
                    location,
                    value: values::ValueType::Decimal(
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
                    let arguments = values::Value::parse_call_params(parser)?;

                    return Ok(Ast::Value {
                        node: values::Value {
                            location,
                            value: values::ValueType::Call {
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
                        node: values::Value {
                            location,
                            value: values::ValueType::Identifier(identifier),
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
                    let components = values::Value::parse_struct(parser)?;

                    return Ok(Ast::Value {
                        node: values::Value {
                            location,
                            value: values::ValueType::Struct {
                                identifier,
                                components,
                            },
                        },
                    });
                }

                Ok(Ast::Value {
                    node: values::Value {
                        location,
                        value: values::ValueType::Identifier(identifier),
                    },
                })
            }

            Lexem::LParen => {
                parser.consume_token(Lexem::LParen)?;

                /* If parsed `()` then we return empty tuple */
                if parser.peek_token().lexem == Lexem::RParen {
                    parser.consume_token(Lexem::RParen)?;
                    return Ok(Ast::Value {
                        node: values::Value {
                            location,
                            value: values::ValueType::Tuple {
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
                    node: values::Value {
                        location,
                        value: values::ValueType::Tuple { components },
                    },
                })
            }

            Lexem::Lsb => values::Value::parse_array(parser),

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
                        node: values::Value {
                            location,
                            value: values::ValueType::Call {
                                identifier: func_id,
                                arguments: vec![
                                    values::CallParam::Positional(0, lhs.clone()),
                                    values::CallParam::Positional(1, rhs.clone()),
                                ],
                            },
                        },
                    };
                } else {
                    let rhs_type = if let Ast::Value {
                        node:
                            values::Value {
                                value: values::ValueType::Identifier(id),
                                ..
                            },
                    } = rhs.as_ref()
                    {
                        types::Type::from_id(id)
                    } else {
                        types::Type::new()
                    };
                    *expr_node = Ast::Expression {
                        node: Box::new(Self {
                            location,
                            expr: ExpressionType::Binary {
                                operation: Lexem::KwAs,
                                lhs: lhs.clone(),
                                rhs: Box::new(Ast::TypeDecl { node: rhs_type }),
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
                            values::Value {
                                value: values::ValueType::Identifier(id),
                                ..
                            },
                    } = rhs.clone().as_ref()
                    {
                        rhs = Box::new(Ast::TypeDecl {
                            node: types::Type::from_id(id),
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

impl IAst for Expression {
    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> types::Type {
        match &self.expr {
            ExpressionType::Binary {
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
                        if let Ast::TypeDecl { node } = rhs.as_ref() {
                            node.clone()
                        } else {
                            analyzer
                                .error(Message::new(self.location, "rhs expected to be a type"));
                            types::Type::new()
                        }
                    } else {
                        rhs.get_type(analyzer)
                    };

                    let mut lhs_type = if let Ast::VariableDef { node } = lhs.as_ref() {
                        node.var_type.clone()
                    } else {
                        lhs.get_type(analyzer)
                    };

                    if let types::Type::Auto = &mut lhs_type {
                        lhs_type = rhs_type.clone();
                    }

                    if lhs_type == rhs_type {
                        return rhs_type;
                    }

                    if is_conversion {
                        return rhs_type;
                    }

                    analyzer.error(Message::new(
                        self.location,
                        &format!("Mismatched types {:?} and {:?}", lhs_type, rhs_type),
                    ));

                    types::Type::new()
                }
            },
            ExpressionType::Unary { node, .. } => node.get_type(analyzer),
        }
    }

    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), Message> {
        match &mut self.expr {
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                let is_conversion = *operation == Lexem::KwAs;

                let mut lhs_type = lhs.get_type(analyzer);

                let rhs_type = if is_conversion {
                    if let Ast::TypeDecl { node } = rhs.as_ref() {
                        node.clone()
                    } else {
                        unreachable!();
                    }
                } else {
                    rhs.analyze(analyzer)?;
                    rhs.get_type(analyzer)
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
                    || is_conversion
                {
                    let does_mutate = !is_conversion;
                    if let Ast::VariableDef { node } = lhs.as_mut() {
                        if analyzer
                            .check_identifier_existance(&node.identifier)
                            .is_ok()
                        {
                            return Err(Message::multiple_ids(
                                node.identifier.location,
                                &node.identifier.get_string(),
                            ));
                        }

                        if types::Type::Auto == node.var_type {
                            node.var_type = rhs_type.clone();
                            lhs_type = rhs_type.clone();
                        }

                        if node.var_type != rhs_type {
                            analyzer.error(Message::new(self.location,
                                &format!("Variable \"{}\" defined with type \"{:?}\", but is assigned to \"{:?}\"",
                                    node.identifier,
                                    node.var_type,
                                    rhs_type)));
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
                        match &node.value {
                            values::ValueType::Identifier(id) => {
                                if let Ok(s) = analyzer.check_identifier_existance(id) {
                                    if let SymbolData::VariableDef { is_mutable, .. } = &s.data {
                                        if !*is_mutable && does_mutate {
                                            analyzer.error(Message::new(
                                                self.location,
                                                &format!(
                                                    "Variable \"{}\" is immutable in current scope",
                                                    id
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                            values::ValueType::Integer(..) | values::ValueType::Decimal(..) => {}
                            values::ValueType::Text(..) => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with text in this context",
                            )),
                            values::ValueType::Array { .. } => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with array in this context",
                            )),
                            values::ValueType::Tuple { .. } => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with tuple in this context",
                            )),
                            _ => analyzer.error(Message::new(
                                self.location,
                                "Cannot perform operation with this object",
                            )),
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
                            analyzer.error(Message::new(
                                self.location,
                                &format!("Cannot cast {:?} to {:?}", lhs_type, rhs_type),
                            ));
                        }
                    } else {
                        analyzer.error(Message::new(
                            self.location,
                            &format!("Cannot perform operation with objects with different types: {:?} and {:?}",
                            lhs_type, rhs_type
                        )));
                    }
                }

                Ok(())
            }
            ExpressionType::Unary { node, .. } => node.analyze(analyzer),
        }
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("operation")?;

        match &self.expr {
            ExpressionType::Unary { operation, node } => {
                writer.put_param("style", "unary")?;
                writer.put_param("operation", operation)?;
                node.serialize(writer)?;
            }
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                writer.put_param("style", "binary")?;
                writer.put_param("operation", operation)?;

                lhs.serialize(writer)?;
                rhs.serialize(writer)?;
            }
        }

        writer.end_tag()?;

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        let old_mode = stream.mode;
        stream.mode = CodeGenMode::SourceOnly;

        match &self.expr {
            ExpressionType::Unary { operation, node } => {
                write!(stream, "{}", operation)?;
                node.codegen(stream)?;
            }
            ExpressionType::Binary {
                operation: Lexem::Assign,
                lhs,
                rhs,
            } => {
                lhs.codegen(stream)?;
                write!(stream, " = ")?;
                rhs.codegen(stream)?;
            }
            ExpressionType::Binary {
                operation: Lexem::KwAs,
                lhs,
                rhs,
            } => {
                write!(stream, "((")?;
                rhs.codegen(stream)?;
                write!(stream, ")")?;
                lhs.codegen(stream)?;
                write!(stream, ")")?;
            }
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                // write!(stream, "(")?;
                lhs.codegen(stream)?;
                write!(stream, " {} ", operation)?;
                rhs.codegen(stream)?;
                // write!(stream, ")")?;
            }
        }

        stream.mode = old_mode;
        Ok(())
    }
}

#[test]
fn conversion_test() {
    use crate::ast::values::{Value, ValueType};
    use crate::parser::lexer::Lexer;

    static SRC_TEXT: &str = "45 as f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT, false).unwrap());

    if let Ast::Expression { node } = Expression::parse(&mut parser).unwrap() {
        if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &node.as_ref().expr
        {
            assert_eq!(*operation, Lexem::KwAs);

            assert!(matches!(
                lhs.as_ref(),
                Ast::Value {
                    node: Value {
                        value: ValueType::Integer(45),
                        ..
                    }
                }
            ));

            assert!(matches!(
                rhs.as_ref(),
                Ast::TypeDecl {
                    node: types::Type::F32
                }
            ))
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected expression");
    };
}
