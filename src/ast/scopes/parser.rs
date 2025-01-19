use super::Scope;
use crate::ast::Ast;
use crate::messages::Message;
use crate::parser::{token::Lexem, Parser};

impl Scope {
    pub fn parse_global(parser: &mut Parser) -> Result<Ast, Message> {
        let mut node = Self::new();

        node.parse_global_internal(parser)?;
        node.is_global = true;

        Ok(Ast::Scope { node })
    }

    fn parse_global_internal(&mut self, parser: &mut Parser) -> Result<(), Message> {
        use crate::ast::{aliases, enums, functions, modules, structs, variables, variants};

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

                Lexem::KwStatic => variables::VariableDef::parse(parser),

                Lexem::KwAlias => aliases::AliasDef::parse(parser),

                _ => {
                    parser.skip_until(&[Lexem::EndOfLine]);
                    parser.get_token();

                    parser.error(Message::new(
                        next.location,
                        &format!("Unexpected token \"{}\"", next),
                    ));
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

    pub fn parse_local(parser: &mut Parser) -> Result<Ast, Message> {
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

    pub fn parse_local_internal(parser: &mut Parser) -> Result<Vec<Ast>, Message> {
        use crate::ast::{aliases, branches, enums, expressions, structs, variables, variants};

        let mut children = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            let child = match next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    parser.get_token();
                    continue;
                }

                Lexem::KwLet => variables::VariableDef::parse(parser),

                Lexem::KwEnum => enums::EnumDef::parse(parser),

                Lexem::KwStruct => structs::StructDef::parse(parser),

                Lexem::KwVariant => variants::VariantDef::parse(parser),

                Lexem::KwAlias => aliases::AliasDef::parse(parser),

                Lexem::KwIf => branches::Branch::parse_if(parser),

                Lexem::KwLoop => branches::Branch::parse_loop(parser),

                Lexem::KwWhile => branches::Branch::parse_while(parser),

                // Lexem::KwFor => branches::parse_for(parser),
                Lexem::KwReturn => branches::Return::parse(parser),

                Lexem::KwBreak => branches::Break::parse(parser),

                Lexem::KwContinue => branches::Continue::parse(parser),

                Lexem::Identifier(_) => expressions::Expression::parse(parser),

                Lexem::Lcb => Self::parse_local(parser),

                Lexem::EndOfFile => {
                    return Err(Message::new(next.location, "Unexpected end of file"))
                }

                _ => {
                    parser.skip_until(&[Lexem::EndOfLine]);
                    parser.get_token();

                    parser.error(Message::unexpected_token(next, &[]));
                    continue;
                }
            };

            match child {
                Ok(child) => children.push(child),
                Err(err) => parser.error(err),
            }

            if let Err(err) = parser.consume_new_line() {
                parser.error(err)
            }
        }

        Ok(children)
    }
}
