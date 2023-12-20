use crate::analyzer::SymbolData;
use crate::ast::{scopes, Ast, IAst, Stream};
use crate::error_listener::MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR;
use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::parser::{Id, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
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
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            analyzer.error(&format!(
                "Identifier \"{}\" defined multiple times",
                &self.identifier
            ));
            return Err(MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR);
        }

        analyzer.scope.push(&self.identifier);
        self.body.analyze(analyzer)?;
        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::ModuleDef {
                full_name: vec![self.identifier.clone()],
            }),
        );

        Ok(())
    }

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
