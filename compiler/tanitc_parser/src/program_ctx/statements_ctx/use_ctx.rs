use tanitc_ast::program_ctx::statement_ctx::use_ctx::UseCtx;
use tanitc_ident::Ident;
use tanitc_lexer::token::lexeme::Lexeme;

use crate::{ParseResult, Parser};

impl Parser {
    pub(crate) fn parse_use_ctx(&mut self) -> ParseResult<UseCtx> {
        let use_tkn = self.consume_token(Lexeme::KwUse)?;
        let idents = self.parse_use_internal()?;

        Ok(UseCtx { use_tkn, idents })
    }

    fn parse_use_internal(&mut self) -> ParseResult<Vec<Ident>> {
        let mut ids = Vec::<Ident>::new();

        loop {
            if self.is_next(Lexeme::EndOfLine) {
                break;
            }

            let id = self.parse_name_ctx()?.identifier();
            ids.push(id);
        }

        Ok(ids)
    }
}
