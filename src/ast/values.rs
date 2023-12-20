use crate::ast::{expressions::Expression, types, Ast, GetType, IAst, Stream};
use crate::error_listener::{
    IDENTIFIER_NOT_FOUND_ERROR_STR, UNEXPECTED_TOKEN_ERROR_STR, WRONG_CALL_ARGUMENTS_ERROR_STR,
};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Parser};

use std::io::Write;

use super::types::Type;

#[derive(Clone, PartialEq)]
pub enum CallParam {
    Notified(String, Box<Ast>),
    Positional(usize, Box<Ast>),
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Call {
        identifier: String,
        arguments: Vec<CallParam>,
    },
    Struct {
        identifier: String,
        components: Vec<(String, Ast)>,
    },
    Tuple {
        components: Vec<Ast>,
    },
    Array {
        components: Vec<Ast>,
    },
    Identifier(String),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

impl Value {
    pub fn parse_call(parser: &mut Parser) -> Result<Vec<CallParam>, &'static str> {
        parser.consume_token(TokenType::LParen)?;

        let mut args = Vec::<CallParam>::new();

        let mut i = 0;
        loop {
            let next = parser.peek_token();

            if next.lexem == TokenType::RParen {
                break;
            }

            let expr = Expression::parse(parser)?;

            let param_id = if let Ast::Value {
                node: Self::Identifier(id),
            } = &expr
            {
                if parser.peek_token().lexem == TokenType::Colon {
                    parser.consume_token(TokenType::Colon)?;
                    Some(id.clone())
                } else {
                    None
                }
            } else {
                None
            };

            let param = if let Some(id) = param_id {
                CallParam::Notified(id, Box::new(Expression::parse(parser)?))
            } else {
                CallParam::Positional(i, Box::new(expr))
            };

            args.push(param);

            i += 1;

            let next = parser.peek_token();
            if next.lexem == TokenType::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == TokenType::RParen {
                // end parsing if ')'
                break;
            } else {
                parser.error("Unexpected token when parsing call", next.get_location());
                return Err(UNEXPECTED_TOKEN_ERROR_STR);
            }
        }

        parser.consume_token(TokenType::RParen)?;

        Ok(args)
    }

    pub fn parse_array(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::Lsb)?;

        let mut components = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            if next.lexem == TokenType::Rsb {
                break;
            }
            components.push(Expression::parse(parser)?);

            let next = parser.peek_token();
            if next.lexem == TokenType::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == TokenType::Rsb {
                // end parsing if ']'
                break;
            } else {
                parser.error("Unexpected token when parsing call", next.get_location());
                return Err(UNEXPECTED_TOKEN_ERROR_STR);
            }
        }

        parser.consume_token(TokenType::Rsb)?;

        Ok(Ast::Value {
            node: Value::Array { components },
        })
    }

    pub fn parse_struct(parser: &mut Parser) -> Result<Vec<(String, Ast)>, &'static str> {
        parser.consume_token(TokenType::Lcb)?;

        let mut components = Vec::<(String, Ast)>::new();

        loop {
            let next = parser.peek_token();

            if next.lexem == TokenType::Rcb {
                break;
            }

            let identifier = parser.consume_identifier()?;

            parser.consume_token(TokenType::Colon)?;

            components.push((identifier, Expression::parse(parser)?));

            let next = parser.peek_token();
            if next.lexem == TokenType::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == TokenType::Rcb {
                // end parsing if '}'
                break;
            } else {
                parser.error(
                    "Unexpected token when parsing struct value",
                    next.get_location(),
                );
                return Err(UNEXPECTED_TOKEN_ERROR_STR);
            }
        }

        parser.consume_token(TokenType::Rcb)?;

        Ok(components)
    }
}

impl IAst for Value {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        match self {
            Self::Integer(_) => Ok(()),

            Self::Decimal(_) => Ok(()),

            Self::Text(_) => Ok(()),

            Self::Identifier(id) => {
                if analyzer.check_identifier_existance(id).is_ok() {
                    analyzer.error(&format!("Cannot find \"{}\" in this scope", id));
                    return Err(IDENTIFIER_NOT_FOUND_ERROR_STR);
                }

                Ok(())
            }

            Self::Call { .. } => {
                if analyzer.check_call_args(self).is_err() {
                    analyzer.error("Wrong call arguments");
                    return Err(WRONG_CALL_ARGUMENTS_ERROR_STR);
                }

                Ok(())
            }

            _ => todo!("Analyzer all values"),
        }
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Call {
                identifier,
                arguments,
            } => {
                writeln!(
                    stream,
                    "{}<call name=\"{}\">",
                    put_intent(intent),
                    identifier
                )?;

                for arg in arguments.iter() {
                    match arg {
                        CallParam::Notified(id, expr) => {
                            writeln!(stream, "{}<param name=\"{}\">", put_intent(intent + 1), id)?;

                            expr.traverse(stream, intent + 2)?;
                        }
                        CallParam::Positional(index, expr) => {
                            writeln!(
                                stream,
                                "{}<param index=\"{}\">",
                                put_intent(intent + 1),
                                index
                            )?;

                            expr.traverse(stream, intent + 2)?;
                        }
                    }

                    writeln!(stream, "{}</param>", put_intent(intent + 1))?;
                }

                writeln!(stream, "{}</call>", put_intent(intent))?;
            }
            Self::Struct {
                identifier,
                components,
            } => {
                if components.is_empty() {
                    return writeln!(
                        stream,
                        "{}<struct name=\"{}\"/>",
                        put_intent(intent),
                        identifier
                    );
                }

                writeln!(
                    stream,
                    "{}<struct name=\"{}\">",
                    put_intent(intent),
                    identifier
                )?;

                for comp in components.iter() {
                    writeln!(
                        stream,
                        "{}<field name=\"{}\">",
                        put_intent(intent + 1),
                        comp.0
                    )?;

                    comp.1.traverse(stream, intent + 2)?;

                    writeln!(stream, "{}</field>", put_intent(intent + 1))?;
                }

                writeln!(stream, "{}</struct>", put_intent(intent))?;
            }
            Self::Tuple { components } => {
                if components.is_empty() {
                    return writeln!(stream, "{}<tuple/>", put_intent(intent));
                }

                writeln!(stream, "{}<tuple>", put_intent(intent))?;

                for comp in components.iter() {
                    comp.traverse(stream, intent + 1)?;
                }

                writeln!(stream, "{}</tuple>", put_intent(intent))?;
            }
            Self::Array { components } => {
                if components.is_empty() {
                    return writeln!(stream, "{}<array/>", put_intent(intent));
                }

                writeln!(stream, "{}<array>", put_intent(intent))?;

                for comp in components.iter() {
                    comp.traverse(stream, intent + 1)?;
                }

                writeln!(stream, "{}</array>", put_intent(intent))?;
            }
            Self::Identifier(id) => {
                writeln!(stream, "{}<variable name=\"{}\"/>", put_intent(intent), id)?
            }
            Self::Text(text) => {
                writeln!(stream, "{}<text content=\"{}\"/>", put_intent(intent), text)?
            }
            Self::Integer(val) => writeln!(
                stream,
                "{}<value type=\"int\" value=\"{}\"/>",
                put_intent(intent),
                val
            )?,
            Self::Decimal(val) => writeln!(
                stream,
                "{}<value type=\"float\" value=\"{}\"/>",
                put_intent(intent),
                val
            )?,
        }

        Ok(())
    }
}

impl GetType for Value {
    fn get_type(&self) -> types::Type {
        match self {
            Self::Text(_) => Type::Ref {
                is_mut: false,
                ref_to: Box::new(Type::Str),
            },
            Self::Decimal(_) => Type::F32,
            Self::Integer(_) => Type::I32,
            _ => todo!("Implement other values get_type"),
        }
    }
}

impl GetType for CallParam {
    fn get_type(&self) -> types::Type {
        match self {
            Self::Notified(_, expr) | Self::Positional(_, expr) => expr.get_type(),
        }
    }
}
