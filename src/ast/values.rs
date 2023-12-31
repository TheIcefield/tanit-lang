use crate::analyzer::SymbolData;
use crate::ast::{expressions::Expression, types, Ast, IAst, Stream};
use crate::error_listener::{
    IDENTIFIER_NOT_FOUND_ERROR_STR, UNEXPECTED_TOKEN_ERROR_STR, WRONG_CALL_ARGUMENTS_ERROR_STR,
};
use crate::lexer::TokenType;
use crate::parser::{put_intent, Id, Parser};

use std::io::Write;

use super::types::Type;

#[derive(Clone, PartialEq)]
pub enum CallParam {
    Notified(Id, Box<Ast>),
    Positional(usize, Box<Ast>),
}

impl IAst for CallParam {
    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> types::Type {
        match self {
            Self::Notified(_, expr) | Self::Positional(_, expr) => expr.get_type(analyzer),
        }
    }

    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        Ok(())
    }

    fn traverse(&self, _stream: &mut Stream, _intent: usize) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Call {
        identifier: Id,
        arguments: Vec<CallParam>,
    },
    Struct {
        identifier: Id,
        components: Vec<(Id, Ast)>,
    },
    Tuple {
        components: Vec<Ast>,
    },
    Array {
        components: Vec<Ast>,
    },
    Identifier(Id),
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

    pub fn parse_struct(parser: &mut Parser) -> Result<Vec<(Id, Ast)>, &'static str> {
        parser.consume_token(TokenType::Lcb)?;

        let mut components = Vec::<(Id, Ast)>::new();

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
                if analyzer.check_identifier_existance(id).is_err() {
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

            Self::Struct {
                identifier,
                components: value_comps,
            } => {
                let ss = analyzer.check_identifier_existance(identifier);
                if ss.is_err() {
                    analyzer.error(&format!("Cannot find \"{}\" in this scope", identifier));
                    return Err(IDENTIFIER_NOT_FOUND_ERROR_STR);
                }

                let ss = ss.unwrap();

                if let SymbolData::StructDef {
                    components: struct_comps,
                } = &ss.data
                {
                    if value_comps.len() != struct_comps.len() {
                        analyzer.error(&format!(
                            "Struct \"{}\" consists of {} fields, but {} were supplied",
                            identifier.get_string(),
                            struct_comps.len(),
                            value_comps.len()
                        ));
                        return Err("Struct definition and declarations is different");
                    }

                    for comp_id in 0..value_comps.len() {
                        let value_comp = value_comps.get(comp_id).unwrap();
                        let value_comp_type = value_comp.1.get_type(analyzer);
                        let struct_comp_type = struct_comps.get(comp_id).unwrap();

                        if value_comp_type != *struct_comp_type {
                            analyzer.error(&format!(
                                "Field named \"{}\" is {:?}, but initialized like {:?}",
                                value_comp.0.get_string(),
                                struct_comp_type,
                                value_comp_type
                            ));
                            return Err("Mismatched types during struct initialization");
                        }
                    }
                } else {
                    analyzer.error(&format!(
                        "Cannot find struct named \"{}\" in this scope",
                        identifier
                    ));
                    return Err(IDENTIFIER_NOT_FOUND_ERROR_STR);
                }

                Ok(())
            }

            _ => todo!("Analyzer all values"),
        }
    }

    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> types::Type {
        match self {
            Self::Text(_) => Type::Ref {
                is_mut: false,
                ref_to: Box::new(Type::Str),
            },
            Self::Decimal(_) => Type::F32,
            Self::Integer(_) => Type::I32,
            Self::Identifier(id) => {
                if let Some(ss) = analyzer.get_symbols(id) {
                    for s in ss.iter().rev() {
                        if analyzer.scope.0.starts_with(&s.scope.0) {
                            if let SymbolData::VariableDef { var_type, .. } = &s.data {
                                return var_type.clone();
                            }
                        }
                    }
                }
                analyzer.error(&format!("No variable found with name \"{}\"", id));
                types::Type::Tuple {
                    components: Vec::new(),
                }
            }
            Self::Struct { identifier, .. } => types::Type::Custom(identifier.to_string()),
            _ => todo!("Implement other values get_type"),
        }
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Call {
                identifier,
                arguments,
            } => {
                writeln!(stream, "{}<call {}>", put_intent(intent), identifier)?;

                for arg in arguments.iter() {
                    match arg {
                        CallParam::Notified(id, expr) => {
                            writeln!(stream, "{}<param {}>", put_intent(intent + 1), id)?;

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
                    return writeln!(stream, "{}<struct {}/>", put_intent(intent), identifier);
                }

                writeln!(stream, "{}<struct {}>", put_intent(intent), identifier)?;

                for comp in components.iter() {
                    writeln!(stream, "{}<field {}>", put_intent(intent + 1), comp.0)?;

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
            Self::Identifier(id) => writeln!(stream, "{}<variable {}/>", put_intent(intent), id)?,
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
