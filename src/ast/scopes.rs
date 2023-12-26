use crate::error_listener::UNEXPECTED_END_OF_LINE_ERROR_STR;
use crate::lexer::TokenType;
use crate::parser::Parser;
use crate::{
    ast,
    ast::{Ast, IAst, Stream},
};

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub statements: Vec<Ast>,
    pub is_global: bool,
}

impl Scope {
    pub fn parse_global(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(TokenType::Lcb)?;

        let statements = Self::parse_global_internal(parser)?;

        parser.consume_token(TokenType::Rcb)?;

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
                TokenType::Rcb | TokenType::EndOfFile => {
                    break;
                }

                TokenType::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                TokenType::KwModule => ast::modules::ModuleNode::parse_def(parser)?,

                TokenType::KwFunc => ast::functions::FunctionNode::parse_def(parser)?,

                TokenType::KwStruct => ast::structs::StructNode::parse_def(parser)?,

                TokenType::KwEnum => ast::structs::EnumNode::parse_def(parser)?,

                TokenType::KwStatic => ast::variables::VariableNode::parse_def(parser)?,

                // TokenType::KwExtern => ast::externs::parse(parser)?,
                TokenType::KwAlias => ast::types::Alias::parse_def(parser)?,

                _ => {
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
        parser.consume_token(TokenType::Lcb)?;

        let statements = Self::parse_local_internal(parser)?;

        parser.consume_token(TokenType::Rcb)?;

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
                TokenType::Rcb => break,

                TokenType::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                TokenType::KwLet => ast::variables::VariableNode::parse_def(parser)?,

                TokenType::KwStruct => ast::structs::StructNode::parse_def(parser)?,

                TokenType::KwEnum => ast::structs::EnumNode::parse_def(parser)?,

                TokenType::KwAlias => ast::types::Alias::parse_def(parser)?,

                TokenType::KwIf => ast::branches::Branch::parse_if(parser)?,

                TokenType::KwLoop => ast::branches::Branch::parse_loop(parser)?,

                TokenType::KwWhile => ast::branches::Branch::parse_while(parser)?,

                // TokenType::KwFor => ast::branch_node::parse_for(parser)?,
                TokenType::KwReturn => ast::branches::Return::parse(parser)?,

                TokenType::KwBreak => ast::branches::Break::parse(parser)?,

                TokenType::KwContinue => ast::branches::Continue::parse(parser)?,

                TokenType::Identifier(_) => ast::expressions::Expression::parse(parser)?,

                TokenType::Lcb => Self::parse_local(parser)?,

                TokenType::EndOfFile => {
                    parser.error("Unexpected end of file", next.get_location());
                    return Err(UNEXPECTED_END_OF_LINE_ERROR_STR);
                }

                _ => {
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
            n.analyze(analyzer)?;
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
}
