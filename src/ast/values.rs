use crate::ast::{expressions, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone)]
pub enum Value {
    Call {
        identifier: String,
        arguments: Vec<Ast>,
    },
    Struct {
        identifier: String,
        components: Vec<(String, Box<Ast>)>,
    },
    Tuple {
        components: Vec<Box<Ast>>,
    },
    Array {
        components: Vec<Box<Ast>>,
    },
    Identifier(String),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

impl IAst for Value {
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
                    arg.traverse(stream, intent + 1)?
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

pub fn parse_call(parser: &mut Parser) -> Option<Vec<Ast>> {
    parser.consume_token(TokenType::LParen)?;

    let mut args = Vec::<Ast>::new();

    loop {
        let next = parser.peek_token();

        if next.lexem == TokenType::RParen {
            break;
        }

        let expr = expressions::parse_expression(parser)?;
        args.push(expr);

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
            return None;
        }
    }

    parser.consume_token(TokenType::RParen)?;

    Some(args)
}

pub fn parse_array_value(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::Lsb)?;

    let mut components = Vec::<Box<Ast>>::new();

    loop {
        let next = parser.peek_token();

        if next.lexem == TokenType::Rsb {
            break;
        }
        components.push(Box::new(expressions::parse_expression(parser)?));

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
            return None;
        }
    }

    parser.consume_token(TokenType::Rsb)?;

    Some(Ast::Value {
        node: Value::Array { components },
    })
}

pub fn parse_struct_value(parser: &mut Parser) -> Option<Vec<(String, Box<Ast>)>> {
    parser.consume_token(TokenType::Lcb)?;

    let mut components = Vec::<(String, Box<Ast>)>::new();

    loop {
        let next = parser.peek_token();

        if next.lexem == TokenType::Rcb {
            break;
        }

        let identifier = parser.consume_identifier()?;

        parser.consume_token(TokenType::Colon)?;

        components.push((identifier, Box::new(expressions::parse_expression(parser)?)));

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
            return None;
        }
    }

    parser.consume_token(TokenType::Rcb)?;

    Some(components)
}
