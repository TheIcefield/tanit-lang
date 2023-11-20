use crate::ast::{scopes, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::{Id, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct Node {
    pub identifier: Id,
    pub body: scopes::Scope,
}

impl IAst for Node {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(
            stream,
            "{}<module name=\"{}\">",
            put_intent(intent),
            self.identifier
        )?;

        self.body.traverse(stream, intent)?;

        writeln!(stream, "{}</module>", put_intent(intent))?;

        Ok(())
    }
}

pub fn parse(parser: &mut Parser) -> Option<Ast> {
    let mut node = parse_header(parser)?;

    node.body = parse_body(parser)?;

    Some(Ast::ModuleDef { node })
}

pub fn parse_header(parser: &mut Parser) -> Option<Node> {
    parser.consume_token(TokenType::KwModule)?;

    let identifier = parser.consume_identifier()?;

    Some(Node {
        identifier,
        body: scopes::Scope {
            statements: Vec::<Ast>::new(),
        },
    })
}

pub fn parse_body(parser: &mut Parser) -> Option<scopes::Scope> {
    scopes::parse_global_external(parser)
}
