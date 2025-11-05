use tanitc_ast::program_ctx::type_ctx::named_type_ctx::{GenericCtx, GenericUnitCtx, NamedTypeCtx};
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_named_type_ctx(&mut self) -> ParseResult<NamedTypeCtx> {
        Ok(NamedTypeCtx {
            name_ctx: Box::new(self.parse_name_ctx()?),
            generic_ctx: {
                if self.is_next(Lexeme::Lt) {
                    Some(self.parse_generic_ctx()?)
                } else {
                    None
                }
            },
        })
    }

    fn parse_generic_ctx(&mut self) -> ParseResult<GenericCtx> {
        Ok(GenericCtx {
            lt_tkn: self.consume_token(Lexeme::Lt)?,
            units_ctx: self.parse_generic_units()?,
            gt_tkn: self.consume_token(Lexeme::Gt)?,
        })
    }

    fn parse_generic_units(&mut self) -> ParseResult<Vec<GenericUnitCtx>> {
        let mut units = Vec::<GenericUnitCtx>::new();
        loop {
            units.push(self.parse_generic_unit_ctx()?);

            if self.is_next(Lexeme::Gt) {
                break;
            }
        }

        Ok(units)
    }

    fn parse_generic_unit_ctx(&mut self) -> ParseResult<GenericUnitCtx> {
        Ok(GenericUnitCtx {
            type_ctx: Box::new(self.parse_type_ctx()?),
            comma_tkn: self.consume_token(Lexeme::Comma).ok(),
        })
    }
}
