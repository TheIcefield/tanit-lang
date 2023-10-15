use crate::lexer::TokenType;
use crate::ast::{Ast, IAst, Stream, types};
use crate::parser::put_intent;
use crate::parser::{Id, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub var_type:   types::Node,
    pub is_field:   bool,
    pub is_global:  bool,
    pub is_mutable: bool
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {

        if self.is_field {
            write!(stream, "{}<field ", put_intent(intent))?;
        } else {
            write!(stream, "{}<variable ", put_intent(intent))?;
        }

        writeln!(stream, "name=\"{}\" is_global=\"{}\" is_mutable=\"{}\">",
            self.identifier, self.is_global, self.is_mutable)?;

        self.var_type.traverse(stream, intent + 1)?;

        if self.is_field {
            writeln!(stream, "{}</field>", put_intent(intent))?;
        } else {
            writeln!(stream, "{}</variable>", put_intent(intent))?;
        }

        Ok(())
    }
}

pub fn parse_def_stmt(parser: &mut Parser) -> Option<Ast> {
    let next = parser.peek_token();

    let is_global = match next.lexem {
        TokenType::KwLet => {
            parser.get_token();
            false
        },

        TokenType::KwStatic => {
            parser.get_token();
            true
        },

        _ => {
            parser.error("Unexpected token, 'let' or 'static' allowed", next.location);
            return None;
        }
    };

    let next = parser.peek_token();
    let is_mutable = match next.lexem {
        TokenType::KwMut => {
            parser.get_token();
            true
        },

        TokenType::KwConst => {
            parser.get_token();
            false
        },

        _ => false,
    };

    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Colon)?;

    let var_type = types::parse(parser)?;

    Some(Ast::VarDef { node: Node {
        identifier,
        var_type,
        is_field: false,
        is_global,
        is_mutable}}
    )
}

/* parse function param */
pub fn parse_param(parser: &mut Parser) -> Option<Node> {
    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Colon)?;

    let var_type = types::parse(parser)?;

    Some(Node{
        identifier,
        var_type,
        is_field: false,
        is_global: false,
        is_mutable: true,
    })
}
