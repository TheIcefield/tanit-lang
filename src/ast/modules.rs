use crate::ast::{scopes, Ast, IAst, Stream};
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::{Id, Parser};

use std::io::Write;

#[derive(Clone)]
pub struct ModuleNode {
    pub identifier: Id,
    pub body: Box<Ast>,
}

impl ModuleNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut node = Self::parse_header(parser)?;

        node.body = Box::new(scopes::Scope::parse_global(parser)?);

        Ok(Ast::ModuleDef { node })
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(TokenType::KwModule)?;

        let identifier = parser.consume_identifier()?;

        Ok(Self {
            identifier,
            body: Box::new(Ast::Scope {
                node: scopes::Scope {
                    statements: Vec::new(),
                    is_global: true,
                },
            }),
        })
    }
}

impl IAst for ModuleNode {
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
