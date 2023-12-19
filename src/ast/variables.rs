use crate::ast::{types, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::{Id, Parser};

use std::io::Write;

use super::expressions;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub var_type: types::Type,
    pub is_global: bool,
    pub is_mutable: bool,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<variable name=\"{}\" is_global=\"{}\" is_mutable=\"{}\">",
            put_intent(intent),
            self.identifier,
            self.is_global,
            self.is_mutable
        )?;

        self.var_type.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</variable>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse_def_stmt(parser: &mut Parser) -> Option<Ast> {
    let next = parser.peek_token();

    let is_global = match next.lexem {
        TokenType::KwLet => {
            parser.get_token();
            false
        }

        TokenType::KwStatic => {
            parser.get_token();
            true
        }

        _ => {
            parser.error(
                "Unexpected token. There only \"let\", \"static\", allowed",
                next.location,
            );
            return None;
        }
    };

    let next = parser.peek_token();
    let is_mutable = match next.lexem {
        TokenType::KwMut => {
            parser.get_token();
            true
        }

        TokenType::KwConst => {
            parser.get_token();
            false
        }

        _ => false,
    };

    let identifier = parser.consume_identifier()?;

    let next = parser.peek_token();

    let mut var_type: Option<types::Type> = None;
    let mut rvalue: Option<Ast> = None;

    if TokenType::Colon == next.lexem {
        parser.consume_token(TokenType::Colon)?;

        var_type = Some(types::Type::parse(parser)?);
    }

    let next = parser.peek_token();

    if TokenType::Assign == next.lexem {
        parser.get_token();

        rvalue = expressions::parse_expression(parser);
    }

    if var_type.is_none() && rvalue.is_none() {
        parser.error(
            &format!(
                "Variable {} defined without type. Need to specify type or use with rvalue",
                identifier
            ),
            next.location,
        );
        return None;
    }

    if var_type.is_none() && is_global {
        parser.error(
            &format!(
                "Variable {} defined without type, but marked as static. Need to specify type",
                identifier
            ),
            next.location,
        );
        return None;
    }

    if var_type.is_none() && rvalue.is_some() {
        var_type = Some(types::Type::Custom("rvalue_type".to_string()));
    }

    let var_node = Ast::VariableDef {
        node: Node {
            identifier,
            var_type: var_type.unwrap(),
            is_global,
            is_mutable,
        },
    };

    if let Some(rhs) = rvalue {
        return Some(Ast::Expression {
            node: Box::new(expressions::Expression::Binary {
                operation: TokenType::Assign,
                lhs: Box::new(var_node),
                rhs: Box::new(rhs),
            }),
        });
    }

    Some(var_node)
}

/* parse function param */
pub fn parse_param(parser: &mut Parser) -> Option<Node> {
    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Colon)?;

    let var_type = types::Type::parse(parser)?;

    Some(Node {
        identifier,
        var_type,
        is_global: false,
        is_mutable: true,
    })
}
