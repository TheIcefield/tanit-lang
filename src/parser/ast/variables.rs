use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::{Id, Parser, ast};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub var_type: ast::types::Node,
    pub is_global: bool,
    pub is_mutable: bool
}

impl ast::IAst for Node {
    fn traverse(&self, stream: &mut ast::Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<variable name=\"{}\" is_global=\"{}\" is_mutable=\"{}\">",
                  put_intent(intent), self.identifier, self.is_global, self.is_mutable)?;

        self.var_type.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</variable>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse_def_stmt(parser: &mut Parser) -> Option<ast::Ast> {
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

    let var_type = ast::types::parse(parser)?;

    Some(ast::Ast::VarDef { node: Node {
        identifier,
        var_type,
        is_global,
        is_mutable}}
    )
}

/* parse struct fields or function param */
pub fn parse_param(parser: &mut Parser) -> Option<Node> {
    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Colon)?;

    let var_type = ast::types::parse(parser)?;

    Some(Node{
        identifier,
        var_type,
        is_global: false,
        is_mutable: true,
    })
}
