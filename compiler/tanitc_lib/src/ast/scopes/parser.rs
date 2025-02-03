use super::Scope;
use crate::ast::Ast;
use crate::parser::Parser;

use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

impl Scope {
    pub fn parse_global(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::default();

        node.parse_scope_internal(parser)?;
        node.is_global = true;

        Ok(Ast::from(node))
    }

    pub fn parse_local(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Scope::default();

        parser.consume_token(Lexem::Lcb)?;

        let old_opt = parser.does_ignore_nl();
        parser.set_ignore_nl_option(false);

        node.parse_scope_internal(parser)?;
        node.is_global = false;

        parser.consume_token(Lexem::Rcb)?;

        parser.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    fn parse_scope_internal(&mut self, parser: &mut Parser) -> Result<(), Message> {
        use crate::ast::{
            aliases, branches, enums, expressions, functions, modules, structs, variables, variants,
        };

        loop {
            let next = parser.peek_token();

            let statement = match next.lexem {
                Lexem::Rcb | Lexem::EndOfFile => {
                    break;
                }

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwDef | Lexem::KwModule => modules::ModuleDef::parse(parser),

                Lexem::KwFunc => functions::FunctionDef::parse(parser),

                Lexem::KwEnum => enums::EnumDef::parse(parser),

                Lexem::KwStruct => structs::StructDef::parse(parser),

                Lexem::KwVariant => variants::VariantDef::parse(parser),

                Lexem::KwLet | Lexem::KwStatic => variables::VariableDef::parse(parser),

                Lexem::KwAlias => aliases::AliasDef::parse(parser),

                Lexem::Identifier(_) | Lexem::Integer(_) | Lexem::Decimal(_) => {
                    expressions::Expression::parse(parser)
                }

                Lexem::KwLoop | Lexem::KwWhile | Lexem::KwIf | Lexem::KwElse => {
                    branches::Branch::parse(parser)
                }

                Lexem::KwReturn | Lexem::KwBreak | Lexem::KwContinue => {
                    branches::Interupter::parse(parser)
                }

                Lexem::Lcb => Self::parse_local(parser),

                _ => {
                    parser.skip_until(&[Lexem::EndOfLine]);
                    parser.get_token();

                    parser.error(Message::unexpected_token(next, &[]));
                    continue;
                }
            };

            match statement {
                Ok(child) => self.statements.push(child),
                Err(err) => parser.error(err),
            }
        }

        Ok(())
    }
}
