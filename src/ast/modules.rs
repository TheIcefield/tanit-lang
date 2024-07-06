use crate::analyzer::SymbolData;
use crate::ast::{identifiers::Identifier, scopes::Scope, Ast, IAst, Stream};
use crate::codegen::CodeGenStream;
use crate::error_listener::{self, MANY_IDENTIFIERS_IN_SCOPE_ERROR_STR};
use crate::lexer::{Lexem, Lexer};
use crate::parser::{put_intent, Parser};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct ModuleNode {
    pub identifier: Identifier,
    pub body: Box<Ast>,
}

impl ModuleNode {
    pub fn parse_def(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut node = Self::parse_header(parser)?;

        node.body = Box::new(Scope::parse_global(parser)?);

        Ok(Ast::ModuleDef { node })
    }

    pub fn parse_header(parser: &mut Parser) -> Result<Self, &'static str> {
        parser.consume_token(Lexem::KwModule)?;

        let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

        Ok(Self {
            identifier,
            body: Box::new(Ast::Scope {
                node: Scope {
                    statements: Vec::new(),
                    is_global: true,
                },
            }),
        })
    }

    pub fn parse_ext_module(parser: &mut Parser) -> Result<Ast, &'static str> {
        let mut node = Self::parse_header(parser)?;

        node.body = Self::parse_ext_body(&node.identifier, parser)?;

        Ok(Ast::ModuleDef { node })
    }

    pub fn parse_ext_body(
        identifier: &Identifier,
        parser: &mut Parser,
    ) -> Result<Box<Ast>, &'static str> {
        let identifier = match identifier {
            Identifier::Common(id) => id.clone(),
            Identifier::Complex(..) => unimplemented!(),
        };

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
        path.push_str(&identifier);

        {
            let mut path = path.clone();
            path.push_str(".tt");

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
                                "Error occured during parsing module \"{}\" body",
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
                                "Error occured during parsing module \"{}\" body",
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
        let identifier = match &self.identifier {
            Identifier::Common(id) => id.clone(),
            Identifier::Complex(..) => {
                analyzer.error(&format!(
                    "Expected common identifier, actually complex: {}",
                    self.identifier
                ));
                return Err("Wrong identifier value");
            }
        };

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

        analyzer.scope.push(&identifier);
        self.body.analyze(analyzer)?;
        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::ModuleDef {
                full_name: vec![identifier],
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

    fn codegen(&self, _stream: &mut CodeGenStream) -> std::io::Result<()> {
        unimplemented!()
    }
}
