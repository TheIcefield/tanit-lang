use crate::lexer::TokenType;
use crate::parser::{Id, ast, put_intent, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub value: ast::types::Node,
}

impl ast::IAst for Node {
    fn traverse(&self, stream: &mut ast::Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<alias name=\"{}\">",
            put_intent(intent), self.identifier)?;

        self.value.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</alias>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse(parser: &mut Parser) -> Option<ast::Ast> {
    parser.consume_token(TokenType::KwAlias)?;

    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Assign)?;

    let value = ast::types::parse(parser)?;

    Some(ast::Ast::AliasDef { node: Node { identifier, value } } )
}

