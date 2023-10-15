use crate::lexer::TokenType;
use crate::ast::{Stream, IAst, Ast, types};
use crate::parser::{Id, put_intent, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub value: types::Node,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<alias name=\"{}\">",
            put_intent(intent), self.identifier)?;

        self.value.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</alias>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse(parser: &mut Parser) -> Option<Ast> {
    parser.consume_token(TokenType::KwAlias)?;

    let identifier = parser.consume_identifier()?;

    parser.consume_token(TokenType::Assign)?;

    let value = types::parse(parser)?;

    Some(Ast::AliasDef { node: Node { identifier, value } } )
}

