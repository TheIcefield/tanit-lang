use crate::ast::{scopes, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::{Id, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub body: Box<Ast>,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<module name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        self.body.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</module>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse(parser: &mut Parser) -> Option<Ast> {
    let mut node = parse_header(parser)?;

    node.body = Box::new(parse_body(parser)?);

    Some(Ast::ModuleDef { node })
}

pub fn parse_header(parser: &mut Parser) -> Option<Node> {
    parser.consume_token(TokenType::KwModule)?;

    let identifier = parser.consume_identifier()?;

    Some(Node {
        identifier,
        body: Box::new(Ast::GScope {
            node: scopes::Scope {
                statements: Vec::new(),
            },
        }),
    })
}

pub fn parse_body(parser: &mut Parser) -> Option<Ast> {
    scopes::parse_global_external(parser)
}
