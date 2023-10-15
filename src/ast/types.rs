use crate::lexer::TokenType;
use crate::ast::{IAst, Stream};
use crate::parser::{Id, Parser, put_intent};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub children: Vec<Node>,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        if self.children.is_empty()
        {
            writeln!(stream, "{}<type name=\"{}\"/>", put_intent(intent), self.identifier)?;
            return Ok(())
        }

        writeln!(stream, "{}<type name=\"{}\">", put_intent(intent), self.identifier)?;

        for i in self.children.iter() {
            i.traverse(stream, intent + 1)?;
        }

        writeln!(stream, "{}</type>", put_intent(intent))?;
            

        Ok(())
    }
}

pub fn parse(parser: &mut Parser) -> Option<Node> {
    let identifier = parser.consume_identifier()?;
    let mut children = Vec::<Node>::new();

    if parser.peek_singular().lexem == TokenType::Lt {
        children = parse_template_args(parser)?;
    }

    Some( Node {
        identifier,
        children
    })
}

pub fn parse_template_args(parser: &mut Parser) -> Option<Vec<Node>> {
    parser.consume_token(TokenType::Lt)?;

    let mut children = Vec::<Node>::new();
    loop {
        let child = parse(parser)?;
        children.push(child);

        let next = parser.peek_singular();
        if next.lexem == TokenType::Gt {
            break;
        } else {
            parser.consume_token(TokenType::Comma)?;
        }
    }

    parser.get_singular();

    Some(children)
}
