use tanitc_ast::program_ctx::statement_ctx::{
    attributes_ctx::AttributesCtx, StatementCtx, StatementsCtx,
};
use tanitc_lexer::token::lexeme::Lexeme;
use tanitc_messages::Message;

pub(crate) mod attributes_ctx;
pub(crate) mod block_ctx;
pub(crate) mod branch_ctx;
pub(crate) mod control_flow_ctx;
pub(crate) mod definition_ctx;
pub(crate) mod expression_ctx;
pub(crate) mod extern_ctx;
pub(crate) mod use_ctx;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_statements_ctx(&mut self) -> ParseResult<StatementsCtx> {
        let mut statements = StatementsCtx::default();

        while let Some(next) = self.peek_token() {
            if matches!(next.lexeme_ref(), Lexeme::Rcb) {
                break;
            }

            let statement = if *next.lexeme_ref() != Lexeme::EndOfLine {
                let attrs = match self.parse_attributes_ctx() {
                    Ok(attrs) => attrs,
                    Err(err) => {
                        self.error(err);
                        AttributesCtx::default()
                    }
                };

                match self.parse_statement_ctx(attrs) {
                    Ok(statement) => Some(statement),
                    Err(err) => {
                        self.error(err);
                        None
                    }
                }
            } else {
                self.get_token();
                None
            };

            let nl_tkn = self.consume_new_line().ok();

            statements.statements.push((statement, nl_tkn));
        }

        Ok(statements)
    }

    fn parse_statement_ctx(&mut self, attrs: AttributesCtx) -> ParseResult<StatementCtx> {
        let next = self.peek_token().ok_or(Message::reached_eof())?;

        let item = match next.lexeme_ref() {
            Lexeme::Lcb => self.parse_block_ctx().map(|mut ctx| {
                ctx.attributes_ctx = Box::new(attrs);
                StatementCtx::Block(ctx)
            }),

            Lexeme::KwAlias
            | Lexeme::KwEnum
            | Lexeme::KwStruct
            | Lexeme::KwUnion
            | Lexeme::KwFunc
            | Lexeme::KwVariant
            | Lexeme::KwDef
            | Lexeme::KwModule
            | Lexeme::KwStatic
            | Lexeme::KwConst
            | Lexeme::KwVar
            | Lexeme::KwExtern
            | Lexeme::KwImpl => self.parse_definition_ctx().map(|mut ctx| {
                ctx.set_attributes(attrs);
                StatementCtx::Definition(ctx)
            }),

            Lexeme::KwReturn | Lexeme::KwBreak | Lexeme::KwContinue => {
                self.check_default_attrs(&attrs).map_err(|mut err| {
                    err.text = format!("In {}: {}", next.lexeme_ref(), err.text);
                    err
                })?;
                self.parse_control_flow_ctx().map(StatementCtx::ControlFlow)
            }

            Lexeme::KwLoop | Lexeme::KwWhile | Lexeme::KwIf | Lexeme::KwElse => {
                self.check_default_attrs(&attrs).map_err(|mut err| {
                    err.text = format!("In {}: {}", next.lexeme_ref(), err.text);
                    err
                })?;
                self.parse_branch_ctx().map(StatementCtx::Branch)
            }

            Lexeme::KwUse => {
                self.check_default_attrs(&attrs).map_err(|mut err| {
                    err.text = format!("In use statement: {}", err.text);
                    err
                })?;
                self.parse_use_ctx().map(StatementCtx::Use)
            }

            Lexeme::Identifier(_)
            | Lexeme::Integer(_)
            | Lexeme::Decimal(_)
            | Lexeme::Ampersand
            | Lexeme::Plus
            | Lexeme::Minus
            | Lexeme::Star
            | Lexeme::Not
            | Lexeme::LParen => {
                self.check_default_attrs(&attrs).map_err(|mut err| {
                    err.text = format!("In expression: {}", err.text);
                    err
                })?;
                self.parse_expression_ctx().map(StatementCtx::Expression)
            }

            _ => {
                self.skip_until(&[Lexeme::EndOfLine]);
                self.get_token();

                return Err(Message::unexpected_token(&next, &[]));
            }
        };

        item
    }

    fn check_default_attrs(&self, attrs: &AttributesCtx) -> Result<(), Message> {
        if let Some(pub_tkn) = &attrs.pub_tkn {
            return Err(Message::new(
                pub_tkn.get_location(),
                "Unexpected attribute \"pub\".",
            ));
        }

        if let Some(safe_tkn) = &attrs.safe_tkn {
            return Err(Message::new(
                safe_tkn.get_location(),
                "Unexpected attribute \"safe\".",
            ));
        }

        if let Some(unsafe_tkn) = &attrs.unsafe_tkn {
            return Err(Message::new(
                unsafe_tkn.get_location(),
                "Unexpected attribute \"unsafe\".",
            ));
        }

        Ok(())
    }
}
