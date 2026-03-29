use tanitc_ast::program_ctx::statement_ctx::{
    block_ctx::BlockCtx,
    definition_ctx::{extern_ctx::ExternCtx, DefinitionCtx},
    StatementCtx,
};
use tanitc_hir::hir::definitions::{externs::ExternDef, functions::FunctionDef};
use tanitc_lexer::location::Location;
use tanitc_messages::Message;

use crate::{AstLowResult, AstLowering};

impl AstLowering {
    pub(crate) fn low_extern_def_ctx(&mut self, ctx: &ExternCtx) -> AstLowResult<ExternDef> {
        let location = ctx.extern_tkn.get_location();
        let abi_name = ctx.abi_tkn.to_string();
        let functions = self.low_extern_def_body_ctx(&ctx.body_ctx, location)?;

        Ok(ExternDef {
            location,
            abi_name,
            functions,
        })
    }

    fn low_extern_def_body_ctx(
        &mut self,
        ctx: &BlockCtx,
        location: Location,
    ) -> AstLowResult<Vec<FunctionDef>> {
        let mut functions = Vec::<FunctionDef>::new();

        for (stmt, _) in &ctx.statements_ctx.statements {
            match stmt {
                Some(StatementCtx::Definition(DefinitionCtx::Func(func))) => {
                    match self.low_func_def_ctx(func) {
                        Ok(func_def) => functions.push(func_def),
                        Err(err) => self.error(err),
                    }
                }
                Some(stmt) => self.error(Message::new(
                    location,
                    format!("{} is now allowed in extern", stmt.kind_str()),
                )),
                _ => continue,
            }
        }

        Ok(functions)
    }
}
