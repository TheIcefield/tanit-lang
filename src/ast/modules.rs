use crate::analyzer::SymbolData;
use crate::ast::{scopes, Ast, IAst, Stream};
use crate::error_listener::{self, MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR};
use crate::lexer::{Lexer, TokenType};
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

    pub fn parse_ext_module(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut node = Self::parse_header(parser)?;

        node.body = Self::parse_ext_body(node.identifier.get_str().unwrap(), parser)?;

        Ok(Ast::ModuleDef { node })
    }

    pub fn parse_ext_body(identifier: &str, parser: &mut Parser) -> Result<Box<Ast>, &'static str> {
        let mut path = parser.get_path()?;
        let verbose = parser.is_token_verbose();
        let mut body: Option<Box<Ast>> = None;

        path = path
            .chars()
            .rev()
            .collect::<String>()
            .splitn(2, '/')
            .collect::<Vec<&str>>()[1]
            .chars()
            .rev()
            .collect::<String>();

        path.push('/');
        path.push_str(identifier);

        {
            let mut path = path.clone();
            path.push_str(".tt");

            // println!("Try parse {}", path);

            let lexer = Lexer::from_file(&path, verbose);

            if let Ok(lexer) = lexer {
                let mut parser_int = Parser::new(lexer, error_listener::ErrorListener::new());

                match parser_int.parse() {
                    Err(mut errors) => {
                        for e in errors.take_errors().iter() {
                            parser.push_error(e.to_string())
                        }
                        parser.error(
                            &format!(
                                "Error occured while during parsing module \"{}\" body",
                                identifier
                            ),
                            parser.get_location(),
                        );
                        return Err("Submodule body parsing error");
                    }

                    Ok(node) => {
                        body = Some(Box::new(node));
                    }
                }
            }
        }

        if body.is_none() {
            let mut path = path.clone();
            path.push_str("/mod.tt");

            // println!("Another try parse {}", path);

            let lexer = Lexer::from_file(&path, verbose);

            if let Ok(lexer) = lexer {
                let mut parser_int = Parser::new(lexer, error_listener::ErrorListener::new());

                match parser_int.parse() {
                    Err(mut errors) => {
                        for e in errors.take_errors().iter() {
                            parser.push_error(e.to_string())
                        }
                        parser.error(
                            &format!(
                                "Error occured while during parsing module \"{}\" body",
                                identifier
                            ),
                            parser.get_location(),
                        );
                        return Err("Submodule body parsing error");
                    }

                    Ok(node) => {
                        body = Some(Box::new(node));
                    }
                }
            }
        }

        if body.is_none() {
            parser.error(
                &format!("Not found definition for module \"{}\"", identifier),
                parser.get_location(),
            );
            return Err("Module definition not found");
        }

        Ok(body.unwrap())
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

        analyzer.scope.push(&self.identifier.get_string());
        self.body.analyze(analyzer)?;
        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::ModuleDef {
                full_name: vec![self.identifier.get_string()],
            }),
        );

        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        writeln!(stream, "{}<module {}>", put_intent(intent), self.identifier)?;

        self.body.traverse(stream, intent + 1)?;

        writeln!(stream, "{}</module>", put_intent(intent))?;

        Ok(())
    }
}
