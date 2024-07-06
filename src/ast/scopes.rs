use crate::codegen::CodeGenStream;
use crate::error_listener::UNEXPECTED_END_OF_LINE_ERROR_STR;
use crate::lexer::Lexem;
use crate::parser::Parser;
use crate::{
    ast,
    ast::{Ast, IAst, Stream},
};

use std::io::Write;

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub statements: Vec<Ast>,
    pub is_global: bool,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            is_global: false,
        }
    }

    pub fn parse_global(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(Lexem::Lcb)?;

        let statements = Self::parse_global_internal(parser)?;

        parser.consume_token(Lexem::Rcb)?;

        Ok(Ast::Scope {
            node: Scope {
                statements,
                is_global: true,
            },
        })
    }

    pub fn parse_global_internal(parser: &mut Parser) -> Result<Vec<Ast>, &'static str> {
        let mut children = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            let child = match next.lexem {
                Lexem::Rcb | Lexem::EndOfFile => {
                    break;
                }

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwModule => ast::modules::ModuleNode::parse_def(parser)?,

                Lexem::KwFunc => ast::functions::FunctionNode::parse_def(parser)?,

                Lexem::KwStruct => ast::structs::StructNode::parse_def(parser)?,

                Lexem::KwEnum => ast::structs::EnumNode::parse_def(parser)?,

                Lexem::KwStatic => ast::variables::VariableNode::parse_def(parser)?,

                Lexem::KwDef => {
                    parser.consume_token(Lexem::KwDef)?;

                    let next = parser.peek_token();

                    match next.lexem {
                        Lexem::KwModule => ast::modules::ModuleNode::parse_ext_module(parser)?,

                        _ => {
                            parser.error(
                                &format!("Unexpected token \"{}\" during parsing define", next),
                                next.get_location(),
                            );
                            continue;
                        }
                    }
                }

                Lexem::KwAlias => ast::types::Alias::parse_def(parser)?,

                _ => {
                    parser.skip_until(Lexem::EndOfLine);
                    parser.get_token();

                    parser.error(
                        &format!("Unexpected token \"{}\"", next),
                        next.get_location(),
                    );
                    continue;
                }
            };

            children.push(child);
        }

        Ok(children)
    }

    pub fn parse_local(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(Lexem::Lcb)?;

        let old_opt = parser.does_ignore_nl();
        parser.set_ignore_nl_option(false);
        let statements = Self::parse_local_internal(parser)?;

        parser.consume_token(Lexem::Rcb)?;

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::Scope {
            node: Scope {
                statements,
                is_global: false,
            },
        })
    }

    pub fn parse_local_internal(parser: &mut Parser) -> Result<Vec<Ast>, &'static str> {
        let mut children = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            let child = match next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwLet => ast::variables::VariableNode::parse_def(parser)?,

                Lexem::KwStruct => ast::structs::StructNode::parse_def(parser)?,

                Lexem::KwEnum => ast::structs::EnumNode::parse_def(parser)?,

                Lexem::KwAlias => ast::types::Alias::parse_def(parser)?,

                Lexem::KwIf => ast::branches::Branch::parse_if(parser)?,

                Lexem::KwLoop => ast::branches::Branch::parse_loop(parser)?,

                Lexem::KwWhile => ast::branches::Branch::parse_while(parser)?,

                // Lexem::KwFor => ast::branch_node::parse_for(parser)?,
                Lexem::KwReturn => ast::branches::Return::parse(parser)?,

                Lexem::KwBreak => ast::branches::Break::parse(parser)?,

                Lexem::KwContinue => ast::branches::Continue::parse(parser)?,

                Lexem::Identifier(_) => ast::expressions::Expression::parse(parser)?,

                Lexem::Lcb => Self::parse_local(parser)?,

                Lexem::EndOfFile => {
                    parser.error("Unexpected end of file", next.get_location());
                    return Err(UNEXPECTED_END_OF_LINE_ERROR_STR);
                }

                _ => {
                    parser.skip_until(Lexem::EndOfLine);
                    parser.get_token();

                    parser.error(
                        &format!("Unexpected token \"{}\"", next),
                        next.get_location(),
                    );
                    continue;
                }
            };

            children.push(child);

            parser.consume_new_line()?;
        }

        Ok(children)
    }
}

impl IAst for Scope {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        let cnt = analyzer.counter();
        analyzer.scope.push(&format!("@s.{}", cnt));
        for n in self.statements.iter_mut() {
            let _ = n.analyze(analyzer);
        }
        analyzer.scope.pop();
        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        for stmt in self.statements.iter() {
            stmt.traverse(stream, intent)?;
        }

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        if !self.is_global {
            writeln!(stream, "{{")?;
        }

        for stmt in self.statements.iter() {
            stmt.codegen(stream)?;

            match stmt {
                Ast::Expression { .. }
                | Ast::BreakStmt { .. }
                | Ast::ContinueStmt { .. }
                | Ast::ReturnStmt { .. } => writeln!(stream, ";")?,
                _ => {}
            }
        }

        if !self.is_global {
            writeln!(stream, "}}")?;
        }
        Ok(())
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
